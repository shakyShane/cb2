use std::fmt;

// `Poll` is a type alias for `Result<Async<T>, E>`
use futures::{Future, Stream, Async, Poll, stream};
use futures::future::ok;
use futures::future::lazy;
use std::fmt::Debug;
use futures::sync::mpsc;
use std::process::Command;
use tokio_process::CommandExt;
use std::process::ExitStatus;

#[derive(Debug)]
enum Cmd {
    Begin(String),
    Running,
    Success,
    Failed
}

#[derive(Debug)]
enum Report {
    Begin { id: usize },
    End { id: usize },
    Running { id: usize },
    Error { id: usize },
}

impl Stream for Cmd {
    type Item = Report;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self {
            Cmd::Begin(s) => {
                *self = Cmd::Running;
                Ok(Async::Ready(Some(Report::Begin {id: 1})))
            }
            Cmd::Running => {
                Ok(Async::Ready(Some(Report::Running {id: 1})))
            }
            Cmd::Success => {
                Ok(Async::Ready(None))
            }
            Cmd::Failed => {
                Ok(Async::Ready(None))
            }
        }
    }
}

#[derive(Debug)]
struct Display<T> {
    group: T,
    curr: usize
}

impl<T> Future for Display<T>
    where T: Stream, T::Item: Debug,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<(), T::Error> {
        let value = try_ready!(self.group.poll());
        println!("{:?}", value);
        Ok(Async::Ready(()))
    }
}

fn create_sync(cmd: &'static str) -> Box<Future<Item = ExitStatus, Error = ()> + Send> {
    Box::new(lazy(move || {
        let mut child = Command::new("sh");
        child.arg("-c").arg(cmd.clone());

        match child.status() {
            Ok(status) => {
                Ok(status)
            }
            _ => Err(())
        }
    }))
}

fn create_async(cmd: &'static str) -> Box<Future<Item = ExitStatus, Error = ()> + Send> {
    Box::new(lazy(move || {
        let child = Command::new("sh").arg("-c").arg(cmd).spawn_async();

        // Make sure our child succeeded in spawning and process the result
        child.expect("failed to spawn")
            .map(|status| status)
            .map_err(|e| panic!("failed to wait for exit: {}", e))
    }))
}

pub fn exec() {
    let s1 = create_sync("echo 'hello' && sleep 1");
    let s2 = create_sync("echo 'there'");
    let s3 = create_async("echo 'world' && sleep 2");
    let s4 = create_async("echo 'world' && sleep 2");
    let s5 = create_async("echo 'world' && sleep 2");
    let s6 = create_sync("echo 'there'");

    let items = vec![
        create_sync("echo 'shane' && sleep 2"),
        create_sync("echo 'is here'")
    ];

    let collected = futures::collect(items).map(|output| ());

    tokio::run(collected)
}
