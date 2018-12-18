use futures::future::lazy;
use futures::future::ok;
use futures::sync::mpsc;
use futures::{stream, Async, Future, Poll, Stream};
use std::fmt;
use std::fmt::Debug;
use std::process::Command;
use std::process::ExitStatus;
use tokio_process::CommandExt;
use crate::task::RunMode;

#[derive(Debug, Clone)]
enum Report {
    Begin { id: usize },
    End { id: usize, exit_code: Option<i32> },
    EndGroup { id: usize, reports: Vec<Report> },
    Running { id: usize },
    Error { id: usize },
    ErrorGroup { id: usize, reports: Vec<Report> },
}

struct TaskItem {
    id: usize,
    cmd: String,
    fail: bool,
}
struct TaskGroup {
    id: usize,
    items: Vec<Task>,
    run_mode: RunMode,
    fail: bool,
}

enum Task {
    Item(TaskItem),
    Group(TaskGroup)
}

pub fn exec() {
    let input = Task::Group(TaskGroup{
        id: 1,
        items: vec![
            Task::Item(TaskItem{
                id: 2,
                cmd: "echo 'first' && sleep 2 && echo 'first->>'".into(),
                fail: true
            }),
            Task::Item(TaskItem{
                id: 3,
                cmd: "echo 'second' && sleep 2 && echo 'second->>'".into(),
                fail: true
            }),
            Task::Group(TaskGroup{
                id: 4,
                items: vec![
                    Task::Item(TaskItem{
                        id: 5,
                        cmd: "echo 'third' && sleep 2 && echo 'third->>'".into(),
                        fail: true
                    }),
                ],
                run_mode: RunMode::Series,
                fail: true
            })
        ],
        run_mode: RunMode::Series,
        fail: true
    });

    let mut items = vec![];

    match input {
        Task::Group(group) => {
            items.push(create_seq(group))
        }
        _ => unimplemented!()
    }

    let collected1 = futures::collect(items)
        .map(move |o| {
            for es in o {
                println!("o={:#?}", es);
            }
            ()
        })
        .map_err(|e| {
            println!("e={:#?}", e);
            ()
        });

    tokio::run(collected1);
}

fn create_sync(task: TaskItem) -> Box<Future<Item = Report, Error = Report> + Send> {
    Box::new(lazy(move || {
        let mut child = Command::new("sh");
        child.arg("-c").arg(task.cmd.clone());

        match child.status() {
            Ok(status) => {
                let report = Report::End {
                    id: task.id.clone(),
                    exit_code: status.code(),
                };
                if status.success() {
                    Ok(report)
                } else {
                    if task.fail {
                        Err(report)
                    } else {
                        Ok(report)
                    }
                }
            }
            _ => Err(Report::Error { id: task.id.clone() }),
        }
    }))
}

fn create_async(task: TaskItem) -> Box<Future<Item = Report, Error = Report> + Send> {
    Box::new(lazy(move || {
        let child = Command::new("sh").arg("-c").arg(task.cmd).spawn_async();
        let id_clone = task.id.clone();

        child
            .expect("failed to spawn")
            .map(move |status| Report::End {
                id: id_clone,
                exit_code: status.code(),
            })
            .map_err(move |e| Report::Error { id: id_clone })
    }))
}

fn create_seq(group: TaskGroup) -> Box<Future<Item = Report, Error = Report> + Send> {
    Box::new(lazy(move || {
        let id_clone = group.id.clone();
        let run_mode = group.run_mode.clone();
        let items_mapped = group.items
            .into_iter()
            .enumerate()
            .map(move |(index, item)| {
//                create_sync(item)
                match item {
                    Task::Item(item) => {
                        match run_mode {
                            RunMode::Series => {
                                create_sync(item)
                            }
                            RunMode::Parallel => {
                                create_async(item)
                            }

                        }
                    },
                    Task::Group(group) => create_seq(group)
                }
            });
        futures::collect(items_mapped).map(move |output| {
            let all_valid = output.iter().all(|report| match report {
                Report::End { exit_code, .. } => match exit_code {
                    Some(0) => true,
                    _ => false,
                },
                _ => false,
            });

            if all_valid {
                Report::EndGroup {
                    id: id_clone,
                    reports: output.clone(),
                }
            } else {
                Report::ErrorGroup {
                    id: id_clone,
                    reports: output.clone(),
                }
            }
        })
    }))
}
