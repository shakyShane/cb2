use crate::input::Input;
use crate::input::TaskDef;
use uuid::Uuid;
use std::fmt;
use std::fmt::Formatter;
use ansi_term::Colour::{Blue, Yellow};
use ansi_term::Style;
use crate::archy::Node;
use crate::archy::archy;
use crate::archy::ArchyOpts;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum RunMode {
    Series,
    Parallel,
}

#[derive(Debug, Clone)]
pub struct TaskItem {
    pub id: String,
    pub cmd: String,
    pub fail: bool,
    pub name: Option<Name>,
}

#[derive(Debug, Clone)]
pub struct TaskGroup {
    pub id: String,
    pub items: Vec<Task>,
    pub run_mode: RunMode,
    pub fail: bool,
    pub name: Option<Name>
}

#[derive(Debug, Clone)]
pub enum Task {
    Item(TaskItem),
    Group(TaskGroup),
}

#[derive(Debug, Clone)]
pub enum Name {
    Alias(String),
    String(String),
    Empty,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let output = match self {
            Name::Alias(alias) => format!("@{}: ", alias),
            Name::String(string) => string.to_string(),
            Name::Empty => String::new(),
        };
        write!(f, "{}", output)
    }
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
    group.into_iter().map(|task| {
        match task {
            Task::Item(item) => Node::new(item_display(item), vec![]),
            Task::Group(group) => Node::new(group_name(group), to_archy_nodes(&group.items)),
        }
    }).collect()
}

fn display_name(task: &Task) -> String {
    match task {
        Task::Item(item) => item_display(&item),
        Task::Group(group) => group_name(&group),
    }
}
fn item_display(item: &TaskItem) -> String {
    match item.name.clone() {
        Some(Name::Alias(s)) => {
            format!("{}{}:\n{}", Style::default().bold().paint("@"), Style::default().bold().paint(s), item.cmd)
        },
        Some(Name::String(s)) => {
            format!("{}\n{}", s, item.cmd)
        },
        Some(Name::Empty) | None => item.cmd.to_string(),
    }
}
fn group_name(group: &TaskGroup) -> String {
    let run_mode = match group.run_mode {
        RunMode::Series => format!("<sequence>"),
        RunMode::Parallel => format!("<group>"),
    };
    match group.name.clone() {
        Some(Name::Alias(s)) => {
            format!("{}{} {}", Style::default().bold().paint("@"), Style::default().bold().paint(s), Blue.paint(run_mode))
        },
        Some(Name::String(s)) => {
            format!("{}{}", s, Blue.paint(run_mode))
        },
        Some(Name::Empty) | None => format!("{}", Blue.paint(run_mode)),
    }
}

impl Task {

    pub fn from_string(string: &str, alias: Option<Name>, input: &Input) -> Task {
        match &string[0..1] {
            "@" => Task::get_task_item(&input, &string[1..string.len()]),
            _ => Task::Item(TaskItem {
                fail: false,
                id: uuid(),
                cmd: string.to_string(),
                name: alias
            }),
        }
    }
    pub fn from_seq(seq: Vec<TaskDef>, alias: Option<Name>, run_mode: RunMode, input: &Input) -> Task {
        let seq_items = seq
            .into_iter()
            .map(|seq_item| match seq_item {
                TaskDef::CmdString(s) => Task::from_string(&s, None, &input),
                TaskDef::TaskObj { command, .. } => Task::from_string(&command, None, &input),
                TaskDef::TaskSeq(seq) => Task::from_seq(seq.to_vec(), None, RunMode::Parallel, &input),
                _ => unimplemented!(),
            })
            .collect::<Vec<Task>>();
        Task::Group(TaskGroup {
            id: uuid(),
            items: seq_items,
            run_mode,
            fail: true,
            name: alias
        })
    }
    pub fn generate_series(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| Task::get_task_item(&input, name))
            .collect::<Vec<Task>>();

        let top_level_names = _names.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(", ");
        let top_level_msg = format!("Input: {} ", top_level_names);

        Task::Group(TaskGroup {
            id: uuid(),
            items: parsed,
            run_mode: RunMode::Series,
            fail: true,
            name: Some(Name::String(top_level_msg))
        })
    }
    pub fn generate_par(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| Task::get_task_item(&input, name))
            .collect::<Vec<Task>>();

        let top_level_names = _names.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(", ");
        let top_level_msg = format!("Input: {} ", top_level_names);

        Task::Group(TaskGroup {
            id: uuid(),
            items: parsed,
            run_mode: RunMode::Parallel,
            fail: false,
            name: Some(Name::String(top_level_msg))
        })
    }
    pub fn get_task_item(input: &Input, name: &str) -> Task {
        let alias = Name::Alias(name.to_string());
        input
            .tasks
            .get(name)
            .map(|item| match item {
                TaskDef::TaskSeq(seq) => Task::from_seq(seq.to_vec(), Some(alias), RunMode::Series, &input),
                TaskDef::TaskSeqObj { run_mode, tasks } => {
                    let run_mode_clone = run_mode.clone().unwrap_or(RunMode::Series);
                    Task::from_seq(tasks.to_vec(), Some(alias), run_mode_clone, &input)
                }
                TaskDef::CmdString(s) => Task::from_string(s, Some(alias), &input),
                TaskDef::TaskObj { command, .. } => Task::from_string(command, Some(alias), &input),
            })
            .unwrap()
    }
}

fn uuid() -> String {
    let id = Uuid::new_v4().to_string();
    let slice = &id[0..8];
    slice.to_string()
}
