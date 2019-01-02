extern crate ansi_term;
extern crate env_logger;
extern crate futures;
extern crate tokio;
use cb2_core::exec;
use futures::future::lazy;

use cb2_core::input::Input;
use cb2_core::report::Report;
use cb2_core::task::Task;
use cb2_core::task_lookup::{select, TaskError, TaskLookup};
use futures::future::Future;
use futures::Stream;

use cb2_core::options::Options;
use cb2_core::task::Status;
use std::env;
use std::process;

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
                    Report::Begin { id } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!("[cb2] {} {} started", Status::Started, name);
                        });
                    }
                    Report::End { id } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!("[cb2] {} {}", Status::Ok, name);
                        });
                    }
                    Report::Error { id } => {
                        flat.get(id).map(|task| {
                            let name = task.name();
                            println!("[cb2] {} {}", Status::Err, name);
                        });
                    }
                    _ => { /* noop */ }
                }
            })
            .collect();

        let main = init.map(move |simple_reports| {
            println!("{}", task_tree.clone().get_tree(&simple_reports));
        });

        tokio::spawn(reports.map(|_| ()).map_err(|_| ()));
        tokio::spawn(main.map(|_| ()).map_err(|_| ()));
        Ok(())
    }));

    Ok((input, lookups))
}
