extern crate env_logger;
extern crate futures;
extern crate tokio;
use futures::future::lazy;

use cb2_core::exec;
use cb2_core::input::Input;
use cb2_core::task::Task;
use cb2_core::task_lookup::{select, TaskError, TaskLookup};
use futures::future::Future;
use futures::Stream;

fn main() {
    env_logger::init();
    let args = vec!["build"];

    ::std::process::exit(match Input::read_from_file("cb2/fixtures/cb2.yaml") {
        Ok(input) => match run(input, args) {
            Ok((_input, _lookups)) => 0,
            Err(_e) => {
                eprintln!("{}", _e);
                1
            }
        },
        Err(e) => {
            eprintln!("{}", e.to_string());
            1
        }
    });
}

fn run(input: Input, names: Vec<&str>) -> Result<(Input, Vec<TaskLookup>), TaskError> {
    let lookups = select(&input, &names)?;
    let task_tree = Task::generate_series(&input, &names);
    //    let task_tree = Task::generate_par(&input, &names);

    let (init, report_stream) = exec::exec(task_tree.clone());

    tokio::run(lazy(move || {
        let reports = report_stream
            .inspect(|report| {
                println!("Report: {:?}", report);
            })
            .collect();

        let main = init.map(move |simple_reports| {
            println!("simple_reports={:?}", simple_reports);
            println!("{}", task_tree.clone().get_tree(&simple_reports));
        });

        tokio::spawn(reports.map(|_| ()).map_err(|_| ()));
        tokio::spawn(main.map(|_| ()).map_err(|_| ()));
        Ok(())
    }));

    Ok((input, lookups))
}
