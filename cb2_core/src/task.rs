use crate::input::Input;
use crate::input::TaskDef;
use uuid::Uuid;
use std::fmt;
use std::fmt::Formatter;
use crate::archy::Node;
use crate::archy::archy;
use crate::archy::ArchyOpts;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum RunMode {
    Series,
    Parallel,
}

#[derive(Debug)]
pub struct TaskItem {
    pub id: String,
    pub cmd: String,
    pub fail: bool,
}

#[derive(Debug)]
pub struct TaskGroup {
    pub id: String,
    pub items: Vec<Task>,
    pub run_mode: RunMode,
    pub fail: bool,
}

#[derive(Debug)]
pub enum Task {
    Item(TaskItem),
    Group(TaskGroup),
}

impl fmt::Display for TaskItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.cmd)
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let output = match self {
            Task::Item(item) => item.cmd.clone(),
            Task::Group(group) => {
                format!("{}", archy(&Node::new(group_name(&group), to_archy_nodes(&group.items)), "", &ArchyOpts::new()))
            },
        };
        write!(f, "{}", output)
    }
}

fn to_archy_nodes(group: &Vec<Task>) -> Vec<Node> {
//    group.items.iter().map(|item| {
//        match item {
//            Task::Item(task_item) => vec![Node::new(task_item.cmd.clone(), vec![])],
//            Task::Group(group) => vec![]
//        }
//    }).collect::<Vec<Node>>()
//    vec![
//        Node::new("echo ls", vec![]),
//        Node::new("echo ls", vec![]),
//        Node::new("echo ls", vec![]),
//    ]
    group.into_iter().map(|task| {
        match task {
            Task::Item(item) => Node::new(item.cmd.clone(), vec![]),
            Task::Group(group) => Node::new(group_name(group), to_archy_nodes(&group.items)),
        }
    }).collect()
}

fn display_name(task: &Task) -> String {
    match task {
        Task::Item(item) => item.cmd.clone(),
        Task::Group(group) => group_name(&group),
    }
}

fn group_name(group: &TaskGroup) -> String {
    match group.run_mode {
        RunMode::Series => "[TaskSeq]".to_string(),
        RunMode::Parallel => "[TaskGroup]".to_string(),
    }
}

impl Task {

    pub fn from_string(string: &str, input: &Input) -> Task {
        match &string[0..1] {
            "@" => Task::get_task_item(&input, &string[1..string.len()]),
            _ => Task::Item(TaskItem {
                fail: false,
                id: uuid(),
                cmd: string.to_string(),
            }),
        }
    }
    pub fn from_seq(seq: Vec<TaskDef>, run_mode: RunMode, input: &Input) -> Task {
        let seq_items = seq
            .into_iter()
            .map(|seq_item| match seq_item {
                TaskDef::CmdString(s) => Task::from_string(&s, &input),
                TaskDef::TaskObj { command, .. } => Task::from_string(&command, &input),
                TaskDef::TaskSeq(seq) => Task::from_seq(seq.to_vec(), RunMode::Parallel, &input),
                _ => unimplemented!(),
            })
            .collect::<Vec<Task>>();
        Task::Group(TaskGroup {
            id: uuid(),
            items: seq_items,
            run_mode,
            fail: true,
        })
    }
    pub fn generate_series(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| Task::get_task_item(&input, name))
            .collect::<Vec<Task>>();

        Task::Group(TaskGroup {
            id: uuid(),
            items: parsed,
            run_mode: RunMode::Series,
            fail: true,
        })
    }
    pub fn generate_par(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| Task::get_task_item(&input, name))
            .collect::<Vec<Task>>();

        Task::Group(TaskGroup {
            id: uuid(),
            items: parsed,
            run_mode: RunMode::Parallel,
            fail: false,
        })
    }
    pub fn get_task_item(input: &Input, name: &str) -> Task {
        input
            .tasks
            .get(name)
            .map(|item| match item {
                TaskDef::TaskSeq(seq) => Task::from_seq(seq.to_vec(), RunMode::Series, &input),
                TaskDef::TaskSeqObj { run_mode, tasks } => {
                    let run_mode_clone = run_mode.clone().unwrap_or(RunMode::Series);
                    Task::from_seq(tasks.to_vec(), run_mode_clone, &input)
                }
                TaskDef::CmdString(s) => Task::from_string(s, &input),
                TaskDef::TaskObj { command, .. } => Task::from_string(command, &input),
            })
            .unwrap()
    }
}

fn uuid() -> String {
    let id = Uuid::new_v4().to_string();
    let slice = &id[0..8];
    slice.to_string()
}
