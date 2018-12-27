use cb2_core::exec;
use cb2_core::input::Input;
use cb2_core::task::Task;
use cb2_core::task_lookup::{select, TaskError, TaskLookup};

fn main() {
    let args = vec!["docker"];

    match Input::read_from_file("cb2/fixtures/cb2.yaml") {
        Ok(input) => match run(input, args) {
            Ok((_input, _lookups)) => println!(""),
            Err(_e) => println!(""),
        },
        Err(e) => println!("{}", e.to_string()),
    };
}

fn run(input: Input, names: Vec<&str>) -> Result<(Input, Vec<TaskLookup>), TaskError> {
    let lookups = select(&input, &names)?;
    //    let task_tree = Task::generate_series(&input, &names);
    let task_tree = Task::generate_par(&input, &names);

    let _ = exec::exec(task_tree);

    Ok((input, lookups))
}
