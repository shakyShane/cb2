use cb2_core::task_lookup::select;
use cb2_core::input::Input;
use cb2_core::task_lookup::TaskError;
use cb2_core::task_lookup::TaskLookup;

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

    let lookups = Input::from_str(yaml)
        .map_err(TaskError::Serde)
        .and_then(|input| select(&input, vec!["swagger"]));

    match lookups {
        Err(e) => println!("{}", e),
        Ok(_lookups) => println!("all good"),
    }
}
