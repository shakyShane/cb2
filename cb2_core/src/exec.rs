use crate::task::{RunMode, Task, TaskGroup, TaskItem};
use futures::future::lazy;
use futures::sync::oneshot;
use futures::stream::{iter_ok, iter_result};
use std::process::Command;
use tokio_process::CommandExt;
use std::process::ExitStatus;
use futures::sync::oneshot::{Sender, Receiver};

use futures::Future;
use futures::Stream;
use futures::{future, stream};
use futures::Async;
use futures::future::{Err as FutureErr, err};
use futures::future::Either;

#[derive(Debug, Clone)]
pub enum Report {
    Unknown,
    End { id: usize },
    EndGroup { id: usize, reports: Vec<Report> },
    Error { id: usize },
    ErrorGroup { id: usize, reports: Vec<Report> },
}

pub fn get_item(cmd: impl Into<String>) -> Box<Future<Item=Result<Report, Report>, Error=futures::Canceled> + Send> {
    let cmd_clone = cmd.into();
    Box::new(lazy(||{
        let (tx, rx) = oneshot::channel();
        tokio::spawn(lazy(move || {
            let mut child = Command::new("sh");
            child.arg("-c").arg(cmd_clone);
            match child.status() {
                Ok(s) => {
                    if s.success() {
                        tx.send(Ok(Report::End{id: 0})).expect("should sent one-shot Ok");
                        Ok(())
                    } else {
                        tx.send(Err(Report::Error{id: 0})).expect("should sent one-shot Err");
                        Err(())
                    }
                }
                Err(e) => {
                    panic!("I can't see how or why we'd get here...");
                    Err(())
                }
            }
        }));
        rx
    }))
}

pub fn exec() {
    tokio::run(lazy(|| {

        let items = vec![
            get_item("echo 1 && sleep 2 && echo 2"),
            get_item("echo 3"),
            get_item("echo 4"),
            get_item("echo '---'"),
        ];

        let items_2 = vec![
            get_item("echo '  101'"),
            get_item("echo '  102'"),
            get_item("echo '  103'"),
            get_item("ech '  104'"),
            get_item("echo '  105'"),
            get_item("echo '  106'"),
            get_item("echo '  117'"),
        ];

        let impl_1 = iter_ok(items)
            .for_each(|this_future| {
                this_future
                    .then(|x| {
                        match x {
                            Ok(Ok(s)) => {
//                                println!("success, continuing={:?}", s);
                                Ok(())
                            },
                            Ok(Err(s)) => {
//                                println!("error, terminating sequence {:?}", s);
                                Err(())
                            }
                            Err(_) => {
//                                println!("unknown error, terminating sequence");
                                Err(())
                            }
                        }
                    })
            });

        let impl_2 = futures::collect(items_2)
            .then(|res| {
                let all_valid = match res {
                    Ok(items) => items.into_iter().all(|x| x.is_ok()),
                    Err(_) => false,
                };

                if all_valid {
                    Ok(())
                } else {
                    Err(())
                }
            });

        tokio::spawn(impl_2.and_then(move |x| impl_1).map(|x| ()).map_err(|e| ()));
//        tokio::spawn(chain.map(|x| ()).map_err(|e| ()));

//        let output = vec![
//            item("echo before && sleep 1 && echo after"),
//            item("echos 1"),
//            item("echo 2"),
//        ].iter().map(stream::iter_ok).flatten_stream();

        Ok(())
    }));
}
