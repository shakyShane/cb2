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
