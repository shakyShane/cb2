use crate::task::{RunMode, Task, TaskGroup, TaskItem};
use futures::future::lazy;
use futures::sync::oneshot;
use futures::stream::{iter_ok, iter_result};
use std::process::Command;
use tokio_process::CommandExt;
use std::process::ExitStatus;
use futures::sync::oneshot::Sender;

use futures::Future;
use futures::Stream;
use futures::{future, stream};
use futures::Async;
use futures::sync::oneshot::Receiver;
use futures::future::{Err as FutureErr, err};
use futures::future::Either;

pub fn exec() {
    tokio::run(lazy(|| {
        let get_item = |cmd: &str| {
            let cmd_string = cmd.to_string();
            lazy(move || {
                let (tx, rx) = oneshot::channel();
                tokio::spawn(lazy(move || {
                    let mut child = Command::new("sh");
                    child.arg("-c").arg(cmd_string);
                    match child.status() {
                        Ok(s) => {
                            if s.success() {
                                tx.send(Ok("hello".to_string())).expect("should sent one-shot Ok");
                                Ok(())
                            } else {
                                tx.send(Err("no".to_string())).expect("should sent one-shot Err");
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
            })
        };

        let items = vec![
            get_item("echo 1 && sleep 2 && echo 2"),
            get_item("eco 3"),
            get_item("echo 3"),
        ];
//
//        let f = get_item("echo 1 && sleep 2 && echo 2")
//            .and_then(move |res| get_item("echo shane 3"))
//            .and_then(move |res| get_item("echo shane 4"))
//            .and_then(move |res| get_item("ech"))
//            .and_then(move |res| {
//                if res.is_ok() {
//                    Either::A(get_item("shane"))
//                } else {
//                    println!("Can abort next run here");
//                    Either::B(futures::future::ok(Ok("yep".to_string())))
//                }
//            })
//            .map(|r| ())
//            .map_err(|e| ());

        let impl_1 = iter_ok::<_, ()>(items)
            .for_each(|this_future| {
                this_future
                    .then(|x| {
                        match x {
                            Ok(Ok(s)) => {
                                println!("succes, continuing={:?}", s);
                                Ok(())
                            },
                            Ok(Err(s)) => {
                                println!("error, terminating sequence {:?}", s);
                                Err(())
                            }
                            Err(_) => {
                                println!("unknown error, terminating sequence");
                                Err(())
                            }
                        }
                    })
                    .map(|x: ()| ())
                    .map_err(|_| {
                        println!("did get an error");
                        ()
                    })
            });

//        let impl_2 = futures::collect(items)
//            .map(|items| {
//                println!("all results {:?}", items);
//                ()
//            })
//            .map_err(|e| ());

        tokio::spawn(impl_1);

//        let output = vec![
//            item("echo before && sleep 1 && echo after"),
//            item("echos 1"),
//            item("echo 2"),
//        ].iter().map(stream::iter_ok).flatten_stream();

        Ok(())
    }));
}
