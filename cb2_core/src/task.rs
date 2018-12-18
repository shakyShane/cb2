use crate::input::Input;

#[derive(Debug, Clone)]
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
    pub fn generate(_input: &Input, _names: &Vec<&str>) -> Task {
        Task::Item(TaskItem{
            id: 0,
            cmd: "echo 'hello world'".into(),
            fail: false
        })
    }
}
