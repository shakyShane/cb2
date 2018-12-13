use crate::input::Input;

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

impl Task {
    pub fn generate(input: &Input, names: &Vec<&str>) -> Task {

        let seq = names.iter().for_each(|name| {
            println!("name={}", name);
        });

        Task::Group {
            id: 0,
            run_mode: RunMode::Series,
            items: vec![
                Task::Item {
                    id: 1,
                    command: "ls".to_string(),
                },
                Task::Item {
                    id: 2,
                    command: "sleep 1".to_string(),
                },
            ],
        }
    }
}