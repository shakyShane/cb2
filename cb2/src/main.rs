use cb2_core::task_lookup::select;
use cb2_core::input::Input;
use cb2_core::task_lookup::{TaskError};
use cb2_core::task::{Task};
use cb2_core::task_lookup::TaskLookup;

fn main() {
    let yaml: &str = r#"
    tasks:
        build:
           - ls dist
           - ["@other", "@build:client"]
           - command: ls -la
        other: ls -l
        other2: ["@other", "@sleep 2"]
        swagger:
          command: swagger is here
        build:client: |
          rimraf dist
          webpack --progress -p
    "#;

    match run(yaml, vec!["build"]) {
        Ok((input, lookups)) => println!("All good, lookups = {:#?}", input),
        Err(e) => println!("{}", e),
    }
}

fn run(input: &str, names: Vec<&str>) -> Result<(Input, Vec<TaskLookup>), TaskError> {
    let input = Input::from_str(input).map_err(TaskError::Serde)?;
    let lookups = select(&input, &names)?;
    let _task_tree = Task::generate(&input, &names);
    Ok((input, lookups))
}