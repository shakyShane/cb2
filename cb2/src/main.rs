#![feature(box_syntax)]

extern crate ansi_term;
extern crate env_logger;
extern crate futures;
extern crate tokio;

use cb2_core::exec;
use futures::future::lazy;
use ansi_term::Colour::{Green, Red, Yellow};
use futures::future::Future;
use futures::Stream;
use futures::sync::oneshot;
use std::env;
use std::fmt;
use std::process;
use std::thread;
use std::time::Duration;
use std::process::Command;
use std::process::Stdio;
use shared_child::SharedChild;
use std::sync::Arc;
use std::process::ExitStatus;
use futures::sync::oneshot::{Sender, Receiver};

struct Task {
    id: String,
    cmd: String,
    parent: String,
}

struct TaskParent {
    id: String,
//    children: Vec<String>
}

impl TaskParent {
    pub fn new() -> TaskParent {
        TaskParent{id: "123456".to_string()}
    }
}

fn task (cmd: String, kill_recv: Receiver<String>) -> impl Future<Item=Result<ExitStatus, ExitStatus>, Error=()> {
    box lazy(move || {

        let (tx, rx) = oneshot::channel();
        let cloned_cmd = cmd.clone();

        let mut child_process = Command::new("sh");
        child_process.arg("-c").arg(cloned_cmd);
        child_process.stdin(Stdio::inherit());
        child_process.stdout(Stdio::inherit());

        let shared_child = SharedChild::spawn(&mut child_process).expect("wrapped");
        let child_arc = Arc::new(shared_child);

        let clone_1 = child_arc.clone();
        let clone_2 = child_arc.clone();

        tokio::spawn(lazy(move || {
            match clone_1.wait() {
                Ok(s) => {
                    if s.success() {
                        tx.send(Ok(s))
                            .map_err(|e| {
                                match e {
                                    Ok(s) => println!("Sent status code = {:?}", s),
                                    Err(e) => println!("Error sending status code = {:?}", e)
                                };
                            })
                    } else {
                        tx.send(Ok(s))
                            .map_err(|e| {
                                println!("Error sending err = {:?}", e);
                            })
                    }
                }
                Err(e) => unimplemented!()
            }
        }));

        tokio::spawn(lazy(move || {
            kill_recv
                .map(move |v: _| {
                    clone_2.kill().expect("killed!");
                    ()
                })
                .map_err(|e: _| {
                    ()
                })
        }));

        rx.map_err(|e| { println!("got error"); })
    })
}


fn main() {
    tokio::run(lazy(|| {
//        let (s, r) = unbounded();

        tokio::spawn(lazy(move || {
            let parent = TaskParent::new();
            let (kill_msg_sender, kill_msg_recv) = oneshot::channel();
            let (kill_msg_sender_2, kill_msg_recv_2) = oneshot::channel();
            let t = task("echo 'start 1' && sleep 3 && echo 'end 1'".to_string(), kill_msg_recv);
            let t2 = task("echo 'start 2' && sleep 3 && echo 'end 2'".to_string(), kill_msg_recv_2);
            let items = vec![t, t2];

            tokio::spawn(lazy(move || {
                thread::sleep(Duration::from_secs(1));
                futures::collect(
                    vec![
                        kill_msg_sender.send("DIE!".to_string()),
                        kill_msg_sender_2.send("DIE!".to_string()),
                    ]
                )
                    .map(|m| println!("All messages sent = {:?}", m))
                    .map_err(|e| println!("kill message failure e = {}", e))
            }));

            futures::collect(items)
                .map(|v| {
                    println!("value-received={:?}", v);
                    ()
                })
                .map_err(|_: ()| {
                    println!("e=");
                    ()
                })
        }));

        Ok(())
//        tds.into_iter().for_each(|td| {
//            td();
//        });
//        let mut tear_downs = vec![];
//        let (rx, teardown) = t.run();
//
//        tear_downs.push(teardown);
//        let (rx2, teardown2) = t2.run();
//        tear_downs.push(teardown2);
//
//        println!("Got return values");
//
//        rx.join(rx2).map(|(v1, v2)| {
//            println!("SUCCESS VALUE={:?}", v1);
//            println!("SUCCESS VALUE={:?}", v2);
//        }).map_err(|e| {
//            println!("ERROR VALUE={:?}", e)
//        })
//        .wait();
//        Ok(())
    }))
}