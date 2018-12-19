use crate::task::{RunMode, Task, TaskGroup, TaskItem};
use futures::future::lazy;
use futures::Future;
use std::process::Command;
use tokio_process::CommandExt;
use futures::stream::iter_ok;
use futures::Stream;
use futures::future::Either;

#[derive(Debug, Clone)]
enum Report {
    End { id: usize, exit_code: Option<i32> },
    EndGroup { id: usize, reports: Vec<Report> },
    Error { id: usize },
    ErrorGroup { id: usize, reports: Vec<Report> },
}

pub fn exec(input: Task) {
    let mut items = vec![];

    match input {
        Task::Group(group) => items.push(create_seq(group)),
        Task::Item(item) => items.push(create_sync(item)),
    }

    let collected1 = futures::collect(items)
        .map(move |o| {
            for es in o {
                println!("o={:?}", es);
            }
            ()
        })
        .map_err(|e| {
            println!("e={:?}", e);
            ()
        });

    tokio::run(collected1);
}

fn create_sync(task: TaskItem) -> Box<Future<Item = (), Error = ()> + Send> {
    Box::new(lazy(move || {
        let mut child = Command::new("sh");
        child.arg("-c").arg(task.cmd.clone());

        match child.status() {
            Ok(status) => {
                let report = Report::End {
                    id: task.id.clone(),
                    exit_code: status.code(),
                };
                println!("sync endede, {}", status);
                if status.success() {
                    Ok(())
                } else {
                    println!("item errored");
                    if task.fail {
                        Err(())
                    } else {
                        Err(())
                    }
                }
            }
            _ => {
                println!("some error");
                Err(())
            },
        }
    }))
}

fn create_async(task: TaskItem) -> Box<Future<Item = (), Error = ()> + Send> {
    Box::new(lazy(move || {
        let cmd_clone = task.cmd.clone();
        let cmd_clone2 = task.cmd.clone();
        let child = Command::new("sh").arg("-c").arg(cmd_clone).spawn_async();
        let id_clone = task.id.clone();

        child
            .expect("failed to spawn")
//            .map(move |status| Report::End {
//                id: id_clone,
//                exit_code: status.code(),
//            })
            .map(move |s| {
                println!("async item success {}, {}", cmd_clone2, s);
                ()
            })
//            .map_err(move |_e| Report::Error { id: id_clone })
            .map_err(move |_e| {
                println!("async item errored");
                ()
            })
    }))
}

fn create_seq(group: TaskGroup) -> Box<Future<Item = (), Error = ()> + Send> {
    Box::new(lazy(move || {
        let id_clone = group.id.clone();
        let run_mode = group.run_mode.clone();
        let run_mode2 = group.run_mode.clone();
        let items_mapped =
            group
                .items
                .into_iter()
                .enumerate()
                .map(move |(_index, item)| match item {
                    Task::Item(item) => match run_mode {
                        RunMode::Series => create_sync(item),
                        RunMode::Parallel => create_async(item),
                    },
                    Task::Group(group) => create_seq(group),
                });

        match run_mode2 {
            RunMode::Series => {
                Either::A(iter_ok::<_, ()>(items_mapped).for_each(|f| f))
            }
            RunMode::Parallel => {
                Either::B(futures::collect(items_mapped).map(move |reports| {
                    let all_valid = true;

                    if all_valid {
                        ()
                    } else {
                        ()
                    }
                }))
            }
        }
    }))
}

///
/// Look at a group of reports and determine if
/// they were all successful
///
fn is_valid_group(reports: Vec<Report>) -> bool {
    reports.into_iter().all(|report| match report {
        Report::End { exit_code, .. } => match exit_code {
            Some(0) => true,
            _ => false,
        },
        Report::EndGroup { reports, .. } => is_valid_group(reports),
        _ => false,
    })
}
