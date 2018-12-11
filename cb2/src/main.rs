use cb2_core::task_group::{RunMode, Task};

fn main() {
    let yaml = r#"
    tasks:
        build:
            - ls
            - sleep 1
    "#;

    let g = Task::Group {
        id: 0,
        run_mode: RunMode::Series,
        items: vec![
            Task::Item {
                id: 1,
                command: "ls".to_string()
            },
            Task::Item {
                id: 2,
                command: "sleep 1".to_string()
            }
        ]
    };

    let g2 = Task::select(yaml, vec!["build", "another"]);
//    println!("{:#?}", g);
}
