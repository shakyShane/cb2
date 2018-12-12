use crate::input::Input;
use crate::input::TaskDef;
use from_file::FromFile;

#[derive(Debug, Clone)]
pub enum PathItem {
    String(String),
    Index(usize)
}

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
    Invalid(Vec<Lookup>),
    Serde(serde_yaml::Error),
}

#[derive(Debug)]
pub enum Lookup {
    Found { target: String, path: Vec<PathItem> },
    NotFound { target: String, path: Vec<PathItem> },
}

impl Task {
    pub fn select(input: &str, names: Vec<&str>) -> Result<Vec<Lookup>, TaskError> {
        let parsed_yml: Result<Input, serde_yaml::Error> = serde_yaml::from_str(input);
        match parsed_yml {
            Err(e) => Err(TaskError::Serde(e)),
            Ok(input) => {
                let parsed = names
                    .iter()
                    .map(|n| validate(&input, n, n, vec![]))
                    .collect::<Vec<Lookup>>();

                let all_valid = parsed.iter().all(|lookup| {
                    match lookup {
                        Lookup::Found {..} => true,
                        Lookup::NotFound {..} => false,
                    }
                });

                if all_valid { Ok(parsed) } else { Err(TaskError::Invalid(parsed)) }
            }
        }
    }
}

pub fn validate(input: &Input, target: &str, name: &str, prev_path: Vec<PathItem>) -> Lookup {
    input.tasks.get(name).map_or_else(
        || {
            let mut next_path = prev_path.clone();
            next_path.push(PathItem::String(name.to_string()));
            Lookup::NotFound {
                target: target.to_string(),
                path: next_path,
            }
        },
        |item| {
            let mut next_path = prev_path.clone();
            next_path.push(PathItem::String(name.to_string()));
            match item {
                TaskDef::CmdString(s) => {
                    validate_string(input, target, name, s.to_string(), next_path)
                }
                TaskDef::TaskObj {..} => {
                    Lookup::Found {
                        target: target.to_string(),
                        path: next_path,
                    }
                }
                TaskDef::TaskSeq(seq) => {
                    validate_seq(input, target, name, seq, next_path)
                }
            }
        },
    )
}

fn validate_seq(input: &Input, target: &str, name: &str, seq: &Vec<TaskDef>, path: Vec<PathItem>) -> Lookup {
    let out = seq.iter().enumerate().map(|(index, seq_item)| {
        let mut next_path = path.clone();
        next_path.push(PathItem::Index(index));
        match seq_item {
            TaskDef::CmdString(s) => {
                validate_string(input, target, name, s.to_string(), next_path)
            }
            TaskDef::TaskSeq(seq) => {
                validate_seq(input, target, name, seq, next_path)
            },
            TaskDef::TaskObj {..} => {
                Lookup::Found {
                    target: target.to_string(),
                    path: next_path,
                }
            }
        }
    }).collect::<Vec<Lookup>>();

    let first_fail = out.into_iter().find(|lookup| {
        match lookup {
            Lookup::Found {..} => false,
            Lookup::NotFound {..} => true,
        }
    });

    if first_fail.is_some() {
        first_fail.unwrap()
    } else {
        Lookup::Found {
            target: target.to_string(),
            path,
        }
    }
}

fn validate_string(input: &Input, target: &str, name: &str, s: String, path: Vec<PathItem>) -> Lookup {
    match &s[0..1] {
        "@" => validate(input, target, &s[1..s.len()], path),
        _ => {
            Lookup::Found {
                target: target.to_string(),
                path,
            }
        }
    }
}
