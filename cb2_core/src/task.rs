use crate::input::Input;
use crate::input::TaskDef;
use from_file::FromFile;
use crate::task_lookup::TaskLookup;
use crate::task_lookup::TaskError;


#[derive(Debug)]
pub enum RunMode {
    Series,
    Parallel,
}

#[derive(Debug)]
pub enum Task {
    Item {
        id: usize,
        command: String,
    },
    Group {
        id: usize,
        items: Vec<Task>,
        run_mode: RunMode,
    },
}
