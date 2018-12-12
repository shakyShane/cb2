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
pub enum Lookup {
    Found {
        name: String
    },
    NotFound {
        name: String
    }
}

impl Task {
    pub fn select(input: &str, names: Vec<&str>) -> Result<Task, TaskError> {
        let parsed_yml: Result<Input, serde_yaml::Error> = serde_yaml::from_str(input);
        match parsed_yml {
            Ok(input) => {
                let valid = names.iter().map(|n| validate(&input, n)).collect::<Vec<Lookup>>();
                println!("valid={:?}", valid);
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

pub fn validate(input: &Input, name: &str) -> Lookup {
    input.tasks.get(name).map_or(Lookup::NotFound {name: name.to_string()}, |item| {
        Lookup::Found {
            name: name.to_string()
        }
    })
}