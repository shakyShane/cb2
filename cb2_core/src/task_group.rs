use crate::input::Input;
use from_file::FromFile;
use crate::input::TaskDef;

#[derive(Debug)]
pub enum RunMode {
    Series,
    Parallel
}

#[derive(Debug)]
pub enum Task {
    Item {
        id: usize,
        command: String
    },
    Group {
        id: usize,
        items: Vec<Task>,
        run_mode: RunMode
    }
}

#[derive(Debug)]
pub enum TaskError {
    Invalid
}

#[derive(Debug)]
pub struct Lookup<'a> {
    input: String,
    result: Option<&'a TaskDef>,
}

#[derive(Debug)]
pub enum LookupResult {
    Found,
    NotFound
}

impl Task {
    pub fn select(input: &str, names: Vec<&str>) -> Result<Task, TaskError> {
        let parsed_yml: Result<Input, serde_yaml::Error> = serde_yaml::from_str(input);
        match parsed_yml {
            Ok(input) => {
                println!("-> results   || {:#?}", results);
                println!("-> all valid || {:#?}", valid);
            }
            Err(e) => {
                println!("{:#?}", input);
            }
        };
        Ok(Task::Item {
            id: 0,
            command: "ls".into()
        })
    }
}