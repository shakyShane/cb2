use crate::input::Input;
use crate::input::TaskDef;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum RunMode {
    Series,
    Parallel,
}

#[derive(Debug)]
pub struct TaskItem {
    pub id: usize,
    pub cmd: String,
    pub fail: bool,
}

#[derive(Debug)]
pub struct TaskGroup {
    pub id: usize,
    pub items: Vec<Task>,
    pub run_mode: RunMode,
    pub fail: bool,
}

#[derive(Debug)]
pub enum Task {
    Item(TaskItem),
    Group(TaskGroup),
}

impl Task {
    pub fn generate_series(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| get_item(&input, name))
            .collect::<Vec<Task>>();

        Task::Group(TaskGroup{
            id: 0,
            items: parsed,
            run_mode: RunMode::Series,
            fail: true
        })
    }
    pub fn generate_par(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|name| get_item(&input, name))
            .collect::<Vec<Task>>();

        Task::Group(TaskGroup{
            id: 0,
            items: parsed,
            run_mode: RunMode::Parallel,
            fail: false
        })
    }
}

fn get_item(input: &Input, name: &str) -> Task {
    input.tasks.get(name).map(|item| {
        match item {
            TaskDef::TaskSeq(seq) => {
                let seq_items: Vec<Task> = seq.into_iter().map(|seq_item| {
                    match seq_item {
                        TaskDef::CmdString(s) => {
                            match &s[0..1] {
                                "@" => get_item(&input, &s[1..s.len()]),
                                _ => Task::Item(TaskItem{
                                    fail: false,
                                    id: 1,
                                    cmd: s.to_string()
                                })
                            }
                        },
                        _ => unimplemented!()
                    }
                }).collect();
                Task::Group(TaskGroup{
                    id: 0,
                    items: seq_items,
                    run_mode: RunMode::Series,
                    fail: true
                })
            },
            TaskDef::CmdString(s) => {
                match &s[0..1] {
                    "@" => get_item(&input, &s[1..s.len()]),
                    _ => Task::Item(TaskItem{
                        fail: false,
                        id: 1,
                        cmd: s.to_string()
                    })
                }
            },
            TaskDef::TaskObj { .. } => unimplemented!(),
        }
    }).unwrap()
}
