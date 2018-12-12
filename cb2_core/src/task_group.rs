use crate::input::Input;
use crate::input::TaskDef;
use from_file::FromFile;

#[derive(Debug)]
pub enum RunMode {
    Series,
    Parallel,
}

#[derive(Debug)]
pub enum Task {
    Item {
        id: usize,
        command: String,
    },
    Group {
        id: usize,
        items: Vec<Task>,
        run_mode: RunMode,
    },
}

#[derive(Debug)]
pub enum TaskError {
    Invalid,
}

#[derive(Debug)]
pub enum Lookup {
    Found { name: String, path: Vec<String> },
    NotFound { name: String, path: Vec<String> },
}

impl Task {
    pub fn select(input: &str, names: Vec<&str>) -> Result<Task, TaskError> {
        let parsed_yml: Result<Input, serde_yaml::Error> = serde_yaml::from_str(input);
        match parsed_yml {
            Ok(input) => {
                let valid = names
                    .iter()
                    .map(|n| validate(&input, n, vec![]))
                    .collect::<Vec<Lookup>>();
                println!("valid={:#?}", valid);
            }
            Err(e) => {
                println!("{:#?}", input);
            }
        };
        Ok(Task::Item {
            id: 0,
            command: "ls".into(),
        })
    }
}

pub fn validate(input: &Input, name: &str, prev_path: Vec<String>) -> Lookup {
    input.tasks.get(name).map_or_else(
        || {
            let mut next_path = prev_path.clone();
            next_path.push(name.to_string());
            Lookup::NotFound {
                name: name.to_string(),
                path: next_path,
            }
        },
        |item| {
            let mut next_path = prev_path.clone();
            next_path.push(name.to_string());
            match item {
                TaskDef::CmdString(s) => {
                    match &s[0..1] {
                        "@" => validate(input, &s[1..s.len()], next_path),
                        _ => {
                            Lookup::Found {
                                name: name.to_string(),
                                path: next_path,
                            }
                        }
                    }
                }
//                TaskDef::TaskObj(obj) => {
//
//                }
                TaskDef::TaskSeq(obj) => {
                    Lookup::Found {
                        name: name.to_string(),
                        path: next_path,
                    }
                }
                _ => unimplemented!()
            }
        },
    )
}
