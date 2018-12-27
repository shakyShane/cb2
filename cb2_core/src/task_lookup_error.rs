use crate::task_lookup::PathItem;

pub fn print(target: &String, path: &Vec<PathItem>) -> String {
    format!(
        r#"
Resolution for `{}` failed.

This means that at some point you tried to reference a task name
that doesn't exist in your input.

Here's the path I tried to resolve:

    {}

The error occured because task {} could not be found
    "#,
        target,
        print_path(path),
        path.iter().last().unwrap().to_string()
    )
}

fn print_path(path: &Vec<PathItem>) -> String {
    path.iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>()
        .join(" -> ")
}

#[test]
fn test_print_path() {
    let path_parts = vec![
        PathItem::String("build".into()),
        PathItem::Index(2),
        PathItem::String("echo hello".into()),
    ];
    let printed = print_path(&path_parts);
    println!("{}", printed);
}
