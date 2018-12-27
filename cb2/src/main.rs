extern crate env_logger;
extern crate futures;

use cb2_core::exec;
use cb2_core::input::Input;
use cb2_core::task::Task;
use cb2_core::task_lookup::{select, TaskError, TaskLookup};
use futures::future::Future;

fn main() {
    env_logger::init();
    let args = vec!["build2"];

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

    let r = exec::exec(task_tree.clone()).wait();

    match r {
        Ok(reports) => {
            println!("{}", task_tree.clone().get_tree(&reports));
        }
        Err(e) => {
            println!("err={:?}", e);
        }
    };

    Ok((input, lookups))
}
