use crate::task::RunMode;
use from_file::FromFile;
use std::collections::HashMap;

#[derive(Debug, Deserialize, FromFile)]
pub struct Input {
    pub tasks: HashMap<String, TaskDef>,
}

impl Input {
    pub fn from_str(input: &str) -> Result<Input, serde_yaml::Error> {
        serde_yaml::from_str(input)
    }
    pub fn read_from_file(input: &str) -> Result<Input, from_file::FromFileError> {
        Input::from_file(input)
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum TaskDef {
    TaskObj {
        command: String,
        env: Option<Vec<String>>,
    },
    CmdString(String),
    TaskSeq(Vec<TaskDef>),
    TaskSeqObj {
        tasks: Vec<TaskDef>,
        run_mode: Option<RunMode>,
    },
}
