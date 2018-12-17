use futures::future::lazy;
use futures::future::ok;
use futures::sync::mpsc;
use futures::{stream, Async, Future, Poll, Stream};
use std::fmt;
use std::fmt::Debug;
use std::process::Command;
use std::process::ExitStatus;
use tokio_process::CommandExt;

#[derive(Debug, Clone)]
enum Report {
    Begin { id: usize },
    End { id: usize, exit_code: Option<i32> },
    EndGroup { id: usize, reports: Vec<Report> },
    Running { id: usize },
    Error { id: usize },
    ErrorGroup { id: usize, reports: Vec<Report> },
}

pub fn exec() {
    let items = vec![
        create_seq(3, vec!["slee", "echo 'after'"], false),
        create_seq(4, vec!["echo 'hello'"], false),
    ];

    let collected1 = futures::collect(items)
        .map(move |o| {
            for es in o {
                println!("o={:?}", es);
            }
            ()
        })
        .map_err(|e| {
            println!("e={:?}", e);
            ()
        });

    tokio::run(collected1);
}

fn create_sync(
    id: usize,
    cmd: &'static str,
    fail: bool,
) -> Box<Future<Item = Report, Error = Report> + Send> {
    Box::new(lazy(move || {
        let mut child = Command::new("sh");
        child.arg("-c").arg(cmd.clone());

        match child.status() {
            Ok(status) => {
                let report = Report::End {
                    id,
                    exit_code: status.code(),
                };
                if status.success() {
                    Ok(report)
                } else {
                    if fail {
                        Err(report)
                    } else {
                        Ok(report)
                    }
                }
            }
            _ => Err(Report::Error { id }),
        }
    }))
}

fn create_async(id: usize, cmd: &'static str) -> Box<Future<Item = Report, Error = Report> + Send> {
    Box::new(lazy(move || {
        let child = Command::new("sh").arg("-c").arg(cmd).spawn_async();

        child
            .expect("failed to spawn")
            .map(move |status| Report::End {
                id,
                exit_code: status.code(),
            })
            .map_err(move |e| Report::Error { id })
    }))
}

fn create_seq(
    id: usize,
    items: Vec<&'static str>,
    fail: bool,
) -> Box<Future<Item = Report, Error = Report> + Send> {
    Box::new(lazy(move || {
        let items_mapped = items
            .into_iter()
            .enumerate()
            .map(move |(index, item)| create_sync(index, item, fail));
        futures::collect(items_mapped).map(move |output| {
            let all_valid = output.iter().all(|report| match report {
                Report::End { exit_code, .. } => match exit_code {
                    Some(0) => true,
                    _ => false,
                },
                _ => false,
            });

            if all_valid {
                Report::EndGroup {
                    id,
                    reports: output.clone(),
                }
            } else {
                Report::ErrorGroup {
                    id,
                    reports: output.clone(),
                }
            }
        })
    }))
}
