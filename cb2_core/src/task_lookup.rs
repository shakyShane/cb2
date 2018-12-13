use crate::input::Input;
use crate::input::TaskDef;
use std::fmt;
use crate::task_lookup_error;

#[derive(Debug)]
pub enum TaskError {
    Invalid(Vec<TaskLookup>),
    Serde(serde_yaml::Error),
}

#[derive(Debug)]
pub enum TaskLookup {
    Found { target: String, path: Vec<PathItem> },
    NotFound { target: String, path: Vec<PathItem> },
}

#[derive(Debug, Clone)]
pub enum PathItem {
    String(String),
    Index(usize),
}

impl fmt::Display for PathItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PathItem::String(s) => write!(f, "`{}`", s),
            PathItem::Index(s) => write!(f, "[index: {}]", s),
        }
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskError::Invalid(lookups) => {
                let output = lookups
                    .iter()
                    .map(|l| match l {
                        TaskLookup::NotFound { target, path } => {
                            task_lookup_error::print(target, path)
                        }
                        _ => String::new(),
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                write!(f, "{}", output)
            }
            TaskError::Serde(e) => write!(f, "{}", e),
        }
    }
}

///
/// Select a sequence of tasks based on the YAML input string
///
pub fn select(input: &Input, names: &Vec<&str>) -> Result<Vec<TaskLookup>, TaskError> {
    let parsed = names
        .iter()
        .map(|n| validate(&input, n, n, vec![]))
        .collect::<Vec<TaskLookup>>();

    let all_valid = parsed.iter().all(|lookup| match lookup {
        TaskLookup::Found { .. } => true,
        TaskLookup::NotFound { .. } => false,
    });

    if all_valid {
        Ok(parsed)
    } else {
        Err(TaskError::Invalid(parsed))
    }
}

pub fn validate(input: &Input, target: &str, name: &str, prev_path: Vec<PathItem>) -> TaskLookup {
    input.tasks.get(name).map_or_else(
        || {
            let mut next_path = prev_path.clone();
            next_path.push(PathItem::String(name.to_string()));
            TaskLookup::NotFound {
                target: target.to_string(),
                path: next_path,
            }
        },
        |item| {
            let mut next_path = prev_path.clone();
            next_path.push(PathItem::String(name.to_string()));
            match item {
                TaskDef::CmdString(s) => {
                    validate_string(input, target, name, s.to_string(), next_path, None)
                }
                TaskDef::TaskObj { .. } => TaskLookup::Found {
                    target: target.to_string(),
                    path: next_path,
                },
                TaskDef::TaskSeq(seq) => validate_seq(input, target, name, seq, next_path),
            }
        },
    )
}

fn validate_seq(
    input: &Input,
    target: &str,
    name: &str,
    seq: &Vec<TaskDef>,
    path: Vec<PathItem>,
) -> TaskLookup {
    let out = seq
        .iter()
        .enumerate()
        .map(|(index, seq_item)| {
            let mut next_path = path.clone();
            next_path.push(PathItem::Index(index));
            match seq_item {
                TaskDef::CmdString(s) => validate_string(
                    input,
                    target,
                    name,
                    s.to_string(),
                    next_path,
                    Some(PathItem::Index(index)),
                ),
                TaskDef::TaskSeq(seq) => validate_seq(input, target, name, seq, next_path),
                TaskDef::TaskObj { .. } => TaskLookup::Found {
                    target: target.to_string(),
                    path: next_path,
                },
            }
        })
        .collect::<Vec<TaskLookup>>();

    let first_fail = out.into_iter().find(|lookup| match lookup {
        TaskLookup::Found { .. } => false,
        TaskLookup::NotFound { .. } => true,
    });

    if first_fail.is_some() {
        first_fail.unwrap()
    } else {
        TaskLookup::Found {
            target: target.to_string(),
            path,
        }
    }
}

fn validate_string(
    input: &Input,
    target: &str,
    name: &str,
    string_input: String,
    path: Vec<PathItem>,
    prepend: Option<PathItem>,
) -> TaskLookup {
    match &string_input[0..1] {
        "@" => validate(input, target, &string_input[1..string_input.len()], path),
        _ => TaskLookup::Found {
            target: target.to_string(),
            path,
        },
    }
}
