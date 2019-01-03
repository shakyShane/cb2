extern crate ansi_term;
extern crate env_logger;
extern crate futures;
extern crate tokio;
use cb2_core::exec;
use futures::future::lazy;

use ansi_term::Colour::{Blue, Green, Red, Yellow};
use cb2_core::input::Input;
use cb2_core::report::Report;
use cb2_core::task::Task;
use cb2_core::task_lookup::{select, TaskError, TaskLookup};
use futures::future::Future;
use futures::Stream;

use cb2_core::options::Options;
use cb2_core::task::Status;
use chrono::format::DelayedFormat;
use chrono::format::StrftimeItems;
use chrono::DateTime;
use chrono::Utc;
use std::env;
use std::fmt;
use std::process;
use cb2_core::report::SimpleReport;
use std::collections::HashMap;

fn main() {
    env_logger::init();
    match Options::from_args(&mut env::args_os()) {
        Ok(options) => {
            process::exit(match Input::read_from_file("cb2.yaml") {
                Ok(input) => match run(input, options.tasks) {
                    Ok((_input, _lookups)) => 0,
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

fn run(input: Input, names: Vec<String>) -> Result<(Input, Vec<TaskLookup>), TaskError> {
    let lookups = select(&input, &names)?;
    let task_tree = Task::generate_series_tree(&input, &names);
    let flat = task_tree.flatten();
    //    println!("{:#?}", flat);
    //    let task_tree = Task::generate_par_tree(&input, &names);

    let (init, report_stream) = exec::exec(task_tree.clone());

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
                    Report::End { id, time, .. } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!(
                                "{} {} {}",
                                Prefix::Ok(time.format("%T").to_string()),
                                Status::Ok,
                                name
                            );
                        });
                    }
                    Report::Error { id, time, .. } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!(
                                "{} {} {}",
                                Prefix::Err(time.format("%T").to_string()),
                                Status::Err,
                                name
                            );
                        });
                    }
                    _ => { /* noop */ }
                }
            })
            .collect();

        let joined = init.join(reports).map(move |(simple_reports, reports)| {
            println!("{}", task_tree.clone().get_tree(&simple_reports));
            println!("{:?}", Report::duration_by_id(reports));
//            println!("{:?}", simple_reports);
//            println!("{:?}", reports);
        });

        tokio::spawn(joined.map(|_: ()| ()).map_err(|_: ()| ()));
        Ok(())
    }));

    Ok((input, lookups))
}
