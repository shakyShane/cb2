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

struct Task {
    id: String,
    cmd: String,
    parent: String
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

impl Runnable for Task {
    fn run(&self) -> (Box<Future<Item=Result<ExitStatus, ExitStatus>, Error=()>>, Box<Fn() -> ()>) {
        let (tx, rx) = oneshot::channel();
        let cloned_cmd = self.cmd.clone();

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
                                println!("err");
                            })
                    } else {
                        tx.send(Err(s))
                            .map_err(|e| {
                                println!("err");
                            })
                    }
                }
                Err(e) => unimplemented!()
            }
        }));

        let teardown = move || {
            clone_2.clone().kill();
        };

        (box rx.map_err(|e| { println!("got error"); }), box teardown)
    }
}

trait Runnable {
    fn run(&self) -> (Box<Future<Item=Result<ExitStatus, ExitStatus>, Error=()>>, Box<Fn() -> ()>);
}

fn main() {
    tokio::run(lazy(|| {
        let parent = TaskParent::new();

        let t = Task{
            id: "01".to_string(),
            cmd: "echo 'start' && sleep 2 && echo 'end'".to_string(),
            parent: parent.id.clone(),
        };

        let t2 = Task{
            id: "02".to_string(),
            cmd: "echo 'start' && sleep 2 && echo 'end'".to_string(),
            parent: parent.id.clone(),
        };

        let mut tear_downs = vec![];
        let (rx, teardown) = t.run();

        tear_downs.push(teardown);
        let (rx2, teardown2) = t2.run();
        tear_downs.push(teardown2);

        println!("Got return values");

        rx.join(rx2).map(|(v1, v2)| {
            println!("SUCCESS VALUE={:?}", v1);
            println!("SUCCESS VALUE={:?}", v2);
        }).map_err(|e| {
            println!("ERROR VALUE={:?}", e)
        })
        .wait();


        Ok(())
    }))
}