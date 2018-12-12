use cb2_core::task_group::{RunMode, Task};

fn main() {
    let yaml = r#"
    tasks:
        build:
           - ls
           - sleep 1
           - "@other"
           - "@other3"
        other: ls -l
        other2: "@other"
    "#;

    let g = Task::Group {
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
    };

    let g2 = Task::select(yaml, vec!["build", "other", "other2"]);
    //    println!("{:#?}", g);
}
