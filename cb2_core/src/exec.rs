use crate::task::{RunMode, Task, TaskGroup, TaskItem};
use futures::future::lazy;
use futures::Future;
use std::process::Command;
use tokio_process::CommandExt;
use futures::Stream;
use futures::stream::iter_ok;
use futures::future::Either;
use futures::Poll;
use futures::Async;
use std::fmt;
use futures::future;
use std::process::ExitStatus;

#[derive(Debug, Clone)]
enum Report {
    End { id: usize, exit_code: Option<i32> },
    EndGroup { id: usize, reports: Vec<Report> },
    Error { id: usize },
    ErrorGroup { id: usize, reports: Vec<Report> },
}
enum State {
    Idle, Running, Done(ExitStatus)
}

struct HelloWorld {
    cmd: String,
    state: State,
    fail: bool
}

impl HelloWorld {
    pub fn new(cmd: impl Into<String>, fail: bool) -> HelloWorld {
        HelloWorld{cmd: cmd.into(), state: State::Idle, fail}
    }
}

impl Future for HelloWorld {
    type Item = Report;
    type Error = Report;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.state {
            State::Idle => {
                let mut child = Command::new("sh");
                child.arg("-c").arg(self.cmd.to_string());
                match child.status() {
                    Ok(s) => {
                        self.state = State::Done(s);
                        Ok(Async::NotReady)
                    }
                    Err(e) => {
                        Err(Report::Error {id: 1})
                    }
                }
            }
            State::Running => {
                Ok(Async::NotReady)
            }
            State::Done(s) => {
                if s.success() {
                    Ok(Async::Ready(Report::End{id: 1, exit_code: Some(0)}))
                } else {
                    if self.fail {
                        Err(Report::Error {id: 1})
                    } else {
                        Ok(Async::Ready(Report::Error{id: 1}))
                    }
                }
            }
        }
    }
}

struct Display<T>(T);

impl<T> Future for Display<T>
    where T: Future, T::Item: fmt::Debug
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<(), T::Error> {
        loop {
            match self.0.poll() {
                Ok(Async::Ready(value)) =>  {
                    return Ok(Async::Ready(()));
                },
                Ok(Async::NotReady) => {
                    // continue polling the child
                },
                Err(err) =>  {
//                    println!("--P-- was err");
                    return Err(err);
                },
            }
        }
    }
}

pub fn exec() {


    tokio::run(lazy(|| {

        let items = vec![
            Display(HelloWorld::new("sleep 1 && echo 1", true)),
            Display(HelloWorld::new("sleep 1 && eco 2", true)),
            Display(HelloWorld::new("sleep 1 && echo 3", true)),
        ];

        println!("Starting");

        let col = futures::collect(items).map(|vec_out| {
            println!("vec-out {:#?}", vec_out);
            ();
        }).map_err(|e| {
            println!("vec-error {:?}", e);
            ()
        });

        tokio::spawn(col);

        Ok(())
    }));
}