use cb2_core::task_group::{RunMode, Task};

fn main() {

    let yaml: &str = r#"
    tasks:
        build:
           - ls dist
           - ["@other", "@builsd:client"]
           - command: ls -la
        other: ls -l
        other2: "@other"
        swagger:
          command: swagger is here
        build:client: |
          rimraf dist
          webpack --progress -p
    "#;

//    let g = Task::Group {
//        id: 0,
//        run_mode: RunMode::Series,
//        items: vec![
//            Task::Item {
//                id: 1,
//                command: "ls".to_string(),
//            },
//            Task::Item {
//                id: 2,
//                command: "sleep 1".to_string(),
//            },
//        ],
//    };

    let g2 = Task::select(yaml, vec!["build", "other", "swagger"]);
    println!("{:#?}", g2);
}
