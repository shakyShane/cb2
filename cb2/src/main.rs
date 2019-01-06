extern crate ansi_term;
extern crate env_logger;
extern crate futures;
extern crate tokio;
use cb2_core::exec;
use futures::future::lazy;

use ansi_term::Colour::{Green, Red, Yellow};
use cb2_core::input::Input;
use cb2_core::report::Report;
use cb2_core::task::Task;
use cb2_core::task_lookup::{select, TaskError, TaskLookup};
use futures::future::Future;
use futures::Stream;

use cb2_core::options::Options;
use cb2_core::task::Dur;
use cb2_core::task::Status;
use futures::sync::oneshot;
use std::env;
use std::fmt;
use std::process;

fn main() {
    env_logger::init();
    match Options::from_args(&mut env::args_os()) {
        Ok(options) => {
            process::exit(match Input::read_from_file("cb2.yaml") {
                Ok(input) => match run(input, options.tasks) {
                    Ok((Ok(_res), _input, _lookups)) => 0,
                    Ok((Err(_res), _input, _lookups)) => 1,
                    Err(e) => {
                        eprintln!("{}", e);
                        1
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    1
                }
            });
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

#[derive(Debug)]
enum Prefix {
    Started(String),
    Ok(String),
    Err(String),
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Prefix::Started(t) => format!("{}", Yellow.paint(format!("[cb2-{}]", t))),
            Prefix::Ok(t) => format!("{}", Green.paint(format!("[cb2-{}]", t))),
            Prefix::Err(t) => format!("{}", Red.paint(format!("[cb2-{}]", t))),
        };
        write!(f, "{}", output)
    }
}

fn run(
    input: Input,
    names: Vec<String>,
) -> Result<(Result<Report, Report>, Input, Vec<TaskLookup>), TaskError> {
    let lookups = select(&input, &names)?;
    let task_tree = Task::generate_series_tree(&input, &names);
    let flat = task_tree.flatten();

    let (init, report_stream) = exec::exec(task_tree.clone());
    let (tx, rx) = oneshot::channel();
    tokio::run(lazy(move || {
        let reports = report_stream
            .inspect(move |report| {
                match report {
                    Report::Started { id, time, .. } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!(
                                "{} {} {}",
                                Prefix::Started(time.format("%T").to_string()),
                                Status::Started,
                                name
                            );
                        });
                    }
                    Report::End { id, time, dur, .. } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!(
                                "{} {} {}",
                                Prefix::Ok(time.format("%T").to_string()),
                                Status::Ok(Dur(dur.clone())),
                                name
                            );
                        });
                    }
                    Report::Error { id, time, dur, .. } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!(
                                "{} {} {}",
                                Prefix::Err(time.format("%T").to_string()),
                                Status::Err(Dur(dur.clone())),
                                name
                            );
                        });
                    }
                    _ => { /* noop */ }
                }
            })
            .collect();

        let joined = init.join(reports).map(move |(init, _reports)| match init {
            Ok(ref report) | Err(ref report) => {
                let flat_reports = report.flatten();
                flat_reports.get(0).map(|report| {
                    let status = match report.clone() {
                        Report::End { dur, .. } | Report::EndGroup { dur, .. } => {
                            Status::Ok(Dur(dur))
                        }
                        Report::Error { dur, .. } | Report::ErrorGroup { dur, .. } => {
                            Status::Err(Dur(dur))
                        }
                        Report::Started { .. } | Report::GroupStarted { .. } => unreachable!(),
                    };

                    match status {
                        Status::Ok(_dur) => println!("\n\tcb2 summary: {}", status),
                        Status::Err(_dur) => {
                            println!("\n\tcb2 summary: {}\n", status);
                            println!("{}", task_tree.clone().get_tree(&report.flatten()));
                        }
                        _ => unimplemented!(),
                    }
                });
                init
            }
        });

        let complete_future = joined
            .inspect(move |report| {
                match tx.send(report.clone()) {
                    Ok(..) => { /* noop */ }
                    Err(e) => {
                        eprintln!("Error from the final send = {:?}", e);
                    }
                }
            })
            .map(|_| ())
            .map_err(|_: ()| ());

        tokio::spawn(complete_future.map(|_| ()).map_err(|_: ()| ()));
        Ok(())
    }));
    let output = rx.wait().expect("output");
    Ok((output, input, lookups))
}
