use futures::future::lazy;
use futures::Future;

use crate::report::Report;
use crate::task::{RunMode, Task};
use crate::task_group::task_group;
use crate::task_item::task_item;
use crate::task_seq::task_seq;
use std::collections::HashMap;

pub type FutureSig = Box<Future<Item = Result<Report, Report>, Error = Report> + Send>;

pub fn exec(task_tree: Task) {
    tokio::run(lazy(move || {

        let as_future = match task_tree.clone() {
            Task::Item(item) => task_item(item),
            Task::Group(group) => match group.run_mode {
                RunMode::Series => task_seq(group),
                RunMode::Parallel => task_group(group),
            },
        };

        let chain = as_future
            .map(|val| {
                println!("outgoing = {:#?}", val);
                ()
            })
            .map_err(move |report| {
//                println!("Err made it to the top = {:#?}", report);
                let as_hashmap = report.simplify();
                println!("{:#?}", as_hashmap);
//                println!("\n\n");
                println!("{:#?}", task_tree);
                println!("{}", task_tree.clone().overlay(as_hashmap));
                ()
            });

        tokio::spawn(chain);

        Ok(())
    }));
}
