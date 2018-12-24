use crate::task::{RunMode, Task, TaskGroup, TaskItem};
use futures::future::lazy;
use futures::sync::oneshot;
use futures::stream::{iter_ok};
use std::process::Command;

use futures::Future;
use futures::Stream;
use std::sync::Arc;
use std::sync::Mutex;

type FutureSig = Box<Future<Item=Result<Report, Report>, Error=()> + Send>;

#[derive(Debug, Clone)]
pub enum Report {
    Unknown,
    End { id: usize },
    EndGroup { id: usize, reports: Vec<Result<Report, Report>> },
    Error { id: usize },
    ErrorGroup { id: usize, reports: Vec<Result<Report, Report>> },
}

pub fn create_item(task_item: TaskItem) -> FutureSig {
    let cmd_clone = task_item.cmd.clone();
    let id_clone = task_item.id.clone();
    Box::new(lazy(move ||{
        let (tx, rx) = oneshot::channel();
        tokio::spawn(lazy(move || {
            let mut child = Command::new("sh");
            child.arg("-c").arg(cmd_clone);
            match child.status() {
                Ok(s) => {
                    if s.success() {
                        tx.send(Ok(Report::End{id: id_clone})).expect("should sent one-shot Ok");
                        Ok(())
                    } else {
                        tx.send(Err(Report::Error{id: id_clone})).expect("should sent one-shot Err");
                        Err(())
                    }
                }
                Err(_e) => {
                    Err(())
                }
            }
        }));
        rx.map_err(|_e| ())
    }))
}

pub fn get_seq(group: TaskGroup) -> FutureSig {
    let id_clone = group.id.clone();
    Box::new(lazy(move || {
        let track: Arc<Mutex<Vec<Result<Report, Report>>>> = Arc::new(Mutex::new(vec![]));
        let c1 = track.clone();
        let c2 = track.clone();

        let items = group.items
            .into_iter()
            .map(|item| {
                match item {
                    Task::Item(item) => create_item(item),
                    Task::Group(group) => match group.run_mode {
                        RunMode::Series => get_seq(group),
                        RunMode::Parallel => get_async_seq(group),
                    },
                }
            });

        iter_ok(items)
            .for_each(move |this_future| {
                let results = c1.clone();
                this_future
                    .then(move |x| {
                        let mut next = results.lock().unwrap();
                        match x {
                            Ok(Ok(s)) => {
                                next.push(Ok(s));
                                Ok(())
                            },
                            Ok(Err(s)) => {
                                next.push(Err(s));
                                Err(())
                            }
                            Err(e) => {
                                Err(())
                            }
                        }
                    })
                    .map(|_e| ())
            })
            .then(move |_res| {

                let next = c2.clone();
                let reports = next.lock().unwrap();
                let all_valid = reports.iter().all(|x| x.is_ok());

                if all_valid {
                    Ok(Ok(Report::EndGroup{id: id_clone, reports: reports.clone()}))
                } else {
                    println!("propagating error :)");
                    Err(())
                }
            })
    }))
}

pub fn get_async_seq(group: TaskGroup) -> FutureSig {
    let id_clone = group.id.clone();
    Box::new(lazy(move || {
        let items = group.items
            .into_iter()
            .map(|item| {
                match item {
                    Task::Item(item) => create_item(item),
                    Task::Group(group) => match group.run_mode {
                        RunMode::Series => get_seq(group),
                        RunMode::Parallel => get_async_seq(group),
                    },
                }
            });

        futures::collect(items)
            .then(move |res| {
                let all_valid = match res {
                    Ok(items) => items.into_iter().all(|x| x.is_ok()),
                    Err(_) => false,
                };

                if all_valid {
                    Ok(Ok(Report::EndGroup{id: id_clone, reports: vec![]}))
                } else {
                    Err(())
                }
            })
    }))
}

pub fn exec(task_tree: Task) {
    tokio::run(lazy(move || {

        let as_future = match task_tree {
            Task::Item(item) => create_item(item),
            Task::Group(group) => match group.run_mode {
                RunMode::Series => get_seq(group),
                RunMode::Parallel => get_async_seq(group),
            }
        };

        let chain = as_future.map(|val| {
            println!("outgoing = {:?}", val);
            ()
        }).map_err(|e| {
            println!("Err made it to the top = {:?}", e);
            ()
        });

        tokio::spawn(chain);
//        tokio::spawn(chain.map(|x| ()).map_err(|e| ()));

//        let output = vec![
//            item("echo before && sleep 1 && echo after"),
//            item("echos 1"),
//            item("echo 2"),
//        ].iter().map(stream::iter_ok).flatten_stream();

        Ok(())
    }));
}
