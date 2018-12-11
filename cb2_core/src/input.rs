use std::collections::HashMap;
use from_file::FromFile;

#[derive(Debug, Deserialize, FromFile)]
pub struct Input {
    pub tasks: HashMap<String, TaskDef>
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TaskDef {
    TaskObj { command: String, env: Option<Vec<String>> },
    CmdString(String),
    TaskSeq(Vec<TaskDef>),
}



