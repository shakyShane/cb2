use crate::input::Input;
use crate::task_lookup::PathItem;
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
    pub fn generate(input: &Input, _names: &Vec<&str>) -> Task {
        let parsed = _names
            .iter()
            .map(|n| get_item(&input, n, vec![]))
            .collect::<Vec<Task>>();

        Task::Group(TaskGroup{
            id: 0,
            items: parsed,
            run_mode: RunMode::Series,
            fail: true
        })
    }
}

fn get_item(input: &Input, name: &str, seen: Vec<PathItem>) -> Task {
    input.tasks.get(name).map(|item| {
        println!("item={:?}", item);
        match item {
            TaskDef::TaskSeq(seq) => {
                Task::Group(TaskGroup{
                    id: 0,
                    items: vec![],
                    run_mode: RunMode::Series,
                    fail: true
                })
            },
            TaskDef::CmdString(string) => {
                Task::Item(TaskItem{
                    id: 1,
                    cmd: string.clone(),
                    fail: true
                })
            },
            TaskDef::TaskObj { .. } => unimplemented!(),
        }
    }).unwrap()
}
