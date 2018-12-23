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

pub fn create_item(cmd: impl Into<String>) -> FutureSig {
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
        rx.map_err(|e| ())
    }))
}

pub fn get_seq(cmds: Vec<&'static str>) -> FutureSig {
    Box::new(lazy(move || {
        let track: Arc<Mutex<Vec<Result<Report, Report>>>> = Arc::new(Mutex::new(vec![]));
        let c1 = track.clone();
        let c2 = track.clone();
        let c3 = track.clone();

        let items = cmds
            .into_iter()
            .map(|cmd| create_item(cmd));

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
                            Err(_) => {
                                unimplemented!()
                            }
                        }
                    })
                    .map(|e| ())
            })
            .then(move |res| {

                let next = c2.clone();
                let mut reports = next.lock().unwrap();
                let all_valid = reports.iter().all(|x| x.is_ok());

                if all_valid {
                    Ok(Ok(Report::EndGroup{id: 1, reports: vec![]}))
                } else {
                    println!("propagating error :)");
                    Err(())
                }
            })
    }))
}

pub fn get_async_seq(cmds: Vec<&'static str>) -> FutureSig {
    Box::new(lazy(move || {
        let items = cmds
            .into_iter()
            .map(|cmd| create_item(cmd));

        futures::collect(items)
            .then(|res| {
                let all_valid = match res {
                    Ok(items) => items.into_iter().all(|x| x.is_ok()),
                    Err(_) => false,
                };

                if all_valid {
                    Ok(Ok(Report::EndGroup{id: 1, reports: vec![]}))
                } else {
                    Err(())
                }
            })
    }))
}

pub fn exec() {
    tokio::run(lazy(|| {

        let s = get_seq(vec!["echo 1"]);
        let s2 = get_async_seq(vec!["echo 1"]);
        let s3 = create_item("echo 3");

        let chain = s.join(s2).join(s3).map(|val| {
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
