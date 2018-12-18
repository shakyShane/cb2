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
    pub fn from_string(string: &str, input: &Input) -> Task {
        match &string[0..1] {
            "@" => Task::get_task_item(&input, &string[1..string.len()]),
            _ => Task::Item(TaskItem {
                fail: false,
                id: 1,
                cmd: string.to_string(),
            }),
        }
    }
    pub fn from_seq(seq: Vec<TaskDef>, run_mode: RunMode, input: &Input) -> Task {
        let seq_items = seq
            .into_iter()
            .map(|seq_item| match seq_item {
                TaskDef::CmdString(s) => Task::from_string(&s, &input),
                TaskDef::TaskObj { command, ..} => Task::from_string(&command, &input),
                TaskDef::TaskSeq(seq) => Task::from_seq(seq.to_vec(), RunMode::Parallel, &input),
                _ => unimplemented!(),
            })
            .collect::<Vec<Task>>();
        Task::Group(TaskGroup {
            id: 0,
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
            id: 0,
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
            id: 0,
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
