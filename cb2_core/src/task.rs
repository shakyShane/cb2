use crate::input::Input;

#[derive(Debug, Clone)]
pub enum RunMode {
    Series,
    Parallel,
}

#[derive(Debug, Clone)]
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

impl Task {
    pub fn generate(input: &Input, names: &Vec<&str>) -> Task {
        Task::Group {
            id: 0,
            run_mode: RunMode::Series,
            items: vec![
                Task::Item {
                    id: 1,
                    command: "slee".to_string(),
                },
                Task::Item {
                    id: 2,
                    command: "ls -l".to_string(),
                },
            ],
        }
    }
}
