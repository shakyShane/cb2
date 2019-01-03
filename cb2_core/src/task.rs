use crate::archy::archy;
use crate::archy::ArchyOpts;
use crate::archy::Node;
use crate::input::Input;
use crate::input::TaskDef;
use crate::report::Report;
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use ansi_term::Style;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum RunMode {
    Series,
    Parallel,
}

impl fmt::Display for RunMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RunMode::Series => write!(f, "{}", Blue.paint("<series>")),
            RunMode::Parallel => write!(f, "{}", Blue.paint("<parallel>")),
        }
    }
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
    pub name: Option<Name>,
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

#[derive(Debug, Clone)]
pub enum Status {
    Ok(Dur),
    Err(Dur),
    NotStarted,
    Started,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Dur(pub chrono::Duration);
impl fmt::Display for Dur {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}s", (self.0.num_milliseconds() as f32) / 1000 as f32)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let output = match self {
            Status::Ok(dur) => {
                let s = format!("✓ ({})", dur);
                format!("{}", Green.paint(s))
            }
            Status::Err(dur) => {
                let s = format!("x ({})", dur);
                format!("{}", Red.paint(s))
            }
            Status::NotStarted => format!("{}", Yellow.paint("-")),
            Status::Started => format!("{}", Yellow.paint("+")),
            Status::Unknown => format!("{}", Yellow.paint("?")),
        };
        write!(f, "{}", output)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let output = match self {
            Name::Alias(alias) => format!("@{}", Style::default().bold().paint(alias)),
            Name::String(string) => string.to_string(),
            Name::Empty => String::new(),
        };
        write!(f, "{}", output)
    }
}

fn to_archy_nodes(group: &Vec<Task>, simple_reports: &Vec<Report>) -> Vec<Node> {
    group
        .into_iter()
        .map(|task| match task {
            Task::Item(item) => Node::new(item_display(item, simple_reports), vec![]),
            Task::Group(group) => Node::new(
                group_name(group, simple_reports),
                to_archy_nodes(&group.items, simple_reports),
            ),
        })
        .collect()
}

fn item_display(item: &TaskItem, simple_reports: &Vec<Report>) -> String {
    let status = item_status(item, simple_reports);
    //    println!("{:#?}", simple_reports);
    //    let dur = Report::duration_by_id(item.id.clone()).unwrap_or(0 as f32);

    match item.name.clone() {
        Some(name) => format!("{} {}\n{}", status, name, item.cmd,),
        None => format!("{} {}", status, item.cmd.to_string()),
    }
}

fn group_name(group: &TaskGroup, simple_reports: &Vec<Report>) -> String {
    let status = group_status(group, simple_reports);
    //    let dur = Report::duration_by_id(group.id.clone(), reports).unwrap_or(0 as f32);
    match group.name.clone() {
        Some(name) => format!("{} {} {}", status, name, group.run_mode),
        None => format!("{} {}", status, group.run_mode),
    }
}

fn group_status(group: &TaskGroup, reports: &Vec<Report>) -> Status {
    reports
        .iter()
        .find(|ref report| report.id() == group.id)
        .map_or(Status::NotStarted, |report| match report {
            Report::EndGroup { dur, .. } => Status::Ok(Dur(dur.clone())),
            Report::ErrorGroup { dur, .. } => Status::Err(Dur(dur.clone())),
            _ => Status::Unknown,
        })
}

fn item_status(task: &TaskItem, reports: &Vec<Report>) -> Status {
    reports
        .iter()
        .find(|report| report.id() == task.id)
        .map_or(Status::NotStarted, |report| match report {
            Report::End { dur, .. } => Status::Ok(Dur(dur.clone())),
            Report::Error { dur, .. } => Status::Err(Dur(dur.clone())),
            _ => Status::Unknown,
        })
}

fn flatten(t: &Task, map: &mut HashMap<String, Task>) {
    match t {
        Task::Item(task_item) => {
            map.insert(task_item.id.clone(), t.clone());
        }
        Task::Group(group) => {
            map.insert(
                group.id.clone(),
                Task::Group(TaskGroup {
                    items: vec![],
                    ..group.clone()
                }),
            );
            group.items.iter().for_each(|item| {
                flatten(item, map);
            })
        }
    }
}

impl Task {
    pub fn name(&self) -> Name {
        match self {
            Task::Item(item) => item
                .name
                .clone()
                .unwrap_or(Name::String(item.cmd.to_string())),
            Task::Group(group) => group
                .name
                .clone()
                .unwrap_or(Name::String("unamed group".into())),
        }
    }
    pub fn flatten(&self) -> HashMap<String, Task> {
        let mut hm = HashMap::new();
        flatten(self, &mut hm);
        hm
    }
    pub fn get_tree(&self, simple_reports: &Vec<Report>) -> String {
        match self {
            Task::Item(item) => item_display(item, simple_reports),
            Task::Group(group) => format!(
                "{}",
                archy(
                    &Node::new(
                        group_name(&group, simple_reports),
                        to_archy_nodes(&group.items, simple_reports)
                    ),
                    "",
                    &ArchyOpts::new()
                )
            ),
        }
    }

    pub fn from_string(string: &str, alias: Option<Name>, input: &Input) -> Task {
        match &string[0..1] {
            "@" => Task::get_task_item(&input, &string[1..string.len()]),
            _ => Task::Item(TaskItem {
                fail: false,
                id: uuid(),
                cmd: string.to_string(),
                name: alias,
            }),
        }
    }
    pub fn from_seq(
        seq: Vec<TaskDef>,
        alias: Option<Name>,
        run_mode: RunMode,
        input: &Input,
    ) -> Task {
        let seq_items = seq
            .into_iter()
            .map(|seq_item| match seq_item {
                TaskDef::CmdString(s) => Task::from_string(&s, None, &input),
                TaskDef::TaskObj { command, .. } => Task::from_string(&command, None, &input),
                TaskDef::TaskSeq(seq) => {
                    Task::from_seq(seq.to_vec(), None, RunMode::Parallel, &input)
                }
                TaskDef::TaskSeqObj { run_mode, tasks } => Task::from_seq(
                    tasks.to_vec(),
                    None,
                    run_mode.unwrap_or(RunMode::Series),
                    &input,
                ),
            })
            .collect::<Vec<Task>>();
        Task::Group(TaskGroup {
            id: uuid(),
            items: seq_items,
            run_mode,
            fail: true,
            name: alias,
        })
    }
    pub fn generate_series_tree(input: &Input, _names: &Vec<String>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| Task::get_task_item(&input, &name))
            .collect::<Vec<Task>>();

        let top_level_names = _names
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let top_level_msg = format!("Input: {} ", Yellow.paint(top_level_names));

        Task::Group(TaskGroup {
            id: uuid(),
            items: parsed,
            run_mode: RunMode::Series,
            fail: true,
            name: Some(Name::String(top_level_msg)),
        })
    }
    pub fn generate_par_tree(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| Task::get_task_item(&input, name))
            .collect::<Vec<Task>>();

        let top_level_names = _names
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let top_level_msg = format!("Input: {} ", top_level_names);

        Task::Group(TaskGroup {
            id: uuid(),
            items: parsed,
            run_mode: RunMode::Parallel,
            fail: false,
            name: Some(Name::String(top_level_msg)),
        })
    }
    pub fn get_task_item(input: &Input, name: &str) -> Task {
        let alias = Name::Alias(name.to_string());
        input
            .tasks
            .get(name)
            .map(|item| match item {
                TaskDef::TaskSeq(seq) => {
                    Task::from_seq(seq.to_vec(), Some(alias), RunMode::Series, &input)
                }
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
