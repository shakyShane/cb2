use cb2_core::exec;
use cb2_core::input::Input;
use cb2_core::task::Task;
use cb2_core::task_lookup::{select, TaskError, TaskLookup};

fn main() {
    let yaml: &str = r#"
    tasks:
        build:
           - ls -l
           - "@other"
        other: echo 'hello world'
    "#;

    match run(yaml, vec!["build"]) {
        Ok((_input, _lookups)) => println!("All good!"),
        Err(e) => println!("{}", e),
    }
}

fn run(input: &str, names: Vec<&str>) -> Result<(Input, Vec<TaskLookup>), TaskError> {
    let input = Input::from_str(input).map_err(TaskError::Serde)?;
    let lookups = select(&input, &names)?;
    let task_tree = Task::generate(&input, &names);
    let _e = exec::exec(task_tree);
    Ok((input, lookups))
}
