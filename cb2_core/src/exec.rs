use crate::task::{RunMode, Task, TaskGroup, TaskItem};
use futures::future::lazy;
use futures::Future;
use std::process::Command;
use tokio_process::CommandExt;
use futures::stream::iter_ok;
use futures::Stream;
use futures::future::join_all;

#[derive(Debug, Clone)]
enum Report {
    End { id: usize, exit_code: Option<i32> },
    EndGroup { id: usize, reports: Vec<Report> },
    Error { id: usize },
    ErrorGroup { id: usize, reports: Vec<Report> },
}

enum G {
    Run(Vec<String>),
    Running,
}

impl Future for G {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Self::Item, Self::Error> {

    }
}

pub fn exec(input: Task) {
//    let mut items = vec![];
//
//    let output = match input {
//        Task::Group(group) => create_seq(group),
//        _ => unimplemented!(),
//    };
//
////    let collected1 = futures::collect(items)
////        .map(move |o| {
////            for es in o {
////                println!("o={:?}", es);
////            }
////            ()
////        })
////        .map_err(|e| {
////            println!("e={:?}", e);
////            ()
////        });
////    let j = join_all(items).then(|x| {
////        println!("x={:?}", x);
////        x
////    });
//    tokio::run(output.and_then(|output| {
//        Ok(())
//    }));
    let async_items = futures::collect(vec![
        create_sync(TaskItem{fail: true, id: 3, cmd: "sleep 2 && echo 3".into()}),
        create_sync(TaskItem{fail: true, id: 4, cmd: "sleep 2 && echo 4".into()}),
    ]).map(|f| ());

//    let items = vec![
//        create_sync(TaskItem{fail: true, id: 1, cmd: "sleep 1 && ech".into()}),
//        Box::new(async_items),
//        create_sync(TaskItem{fail: true, id: 2, cmd: "sleep 1 && echo 2".into()}),
//    ];
//    let collected1 = iter_ok::<_, ()>(items).for_each(|f|f);

    tokio::run(async_items);
}

fn create_sync(task: TaskItem) -> Box<Future<Item = (), Error = ()> + Send> {
    Box::new(lazy(move || {
        let mut child = Command::new("sh");
        child.arg("-c").arg(task.cmd.clone());

        match child.status() {
            Ok(status) => {
//                let report = Report::End {
//                    id: task.id.clone(),
//                    exit_code: status.code(),
//                };
//                if status.success() {
//                    Ok(report)
//                } else {
//                    if task.fail {
//                        Err(report)
//                    } else {
//                        Ok(report)
//                    }
//                }
                Ok(())
            }
            _ => Ok(()),
        }
    }))
}

fn create_async(task: TaskItem) -> Box<Future<Item = (), Error = ()> + Send> {
    Box::new(lazy(move || {
        let child = Command::new("sh").arg("-c").arg(task.cmd).spawn_async();
        let id_clone = task.id.clone();

        child
            .expect("failed to spawn")
            .map(move |status| {
                ()
            })
            .map_err(move |_e| {
                ()
            })
    }))
}

fn create_seq(group: TaskGroup) -> Box<Future<Item = (), Error = ()> + Send> {
    Box::new(lazy(move || {
        let id_clone = group.id.clone();
        let run_mode = group.run_mode.clone();
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
                    _ => unimplemented!()
                });

        futures::collect(items_mapped).map(move |reports| {
            ()
        })
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
