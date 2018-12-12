use cb2_core::task_lookup::{select};

fn main() {

    let yaml: &str = r#"
    tasks:
        build:
           - ls dist
           - ["@other", "@build:client"]
           - command: ls -la
        other: ls -l
        other2: ["@other", "@sleep 2"]
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

    let g2 = select(yaml, vec!["other2"]);

    match g2 {
        Err(e) => println!("{}", e),
        Ok(lookups) => println!("all good")
    }
}
