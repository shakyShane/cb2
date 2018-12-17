use std::fmt;

// `Poll` is a type alias for `Result<Async<T>, E>`
use futures::{Future, Stream, Async, Poll, stream};
use futures::future::ok;
use futures::future::lazy;
use std::fmt::Debug;
use futures::sync::mpsc;
use std::process::Command;
use tokio_process::CommandExt;

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

pub fn exec() {
    // Use the standard library's `Command` type to build a process and
    // then execute it via the `CommandExt` trait.
    let create_async = |cmd: &'static str| {
        Box::new(lazy(move || {
            let child = Command::new("sh").arg("-c").arg(cmd).spawn_async();

            // Make sure our child succeeded in spawning and process the result
            child.expect("failed to spawn")
                .map(|status| status)
                .map_err(|e| panic!("failed to wait for exit: {}", e))
        }))
    };

    let create_sync = |cmd: &'static str| {
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
    };

    let items_async = vec![
        create_async("echo 'hello'"),
        create_async("sleep 1 && echo '2'"),
        create_async("echo 'shane'"),
    ];

    let items_sync = vec![
        create_sync("echo 'hello'"),
        create_sync("sleep 1 && echo '2'"),
        create_sync("echo 'shane'"),
    ];

//    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
//    let s = runtime.block_on(example());
//    println!("{:?}", s);

    let collected = futures::collect(items_sync).map(|output| {
        println!("{:?}", output);
        ()
    });

    // Send the future to the tokio runtime for execution
    tokio::run(
        collected
////        seq
//        stream::iter_ok(items).for_each(|f| f)
    );
}