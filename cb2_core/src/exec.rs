use futures::future::lazy;
use futures::Future;

use crate::report::Report;
use crate::task::{RunMode, Task};
use crate::task_group::task_group;
use crate::task_item::task_item;
use crate::task_seq::task_seq;

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
        let c1 = task_tree.clone();
        let c2 = task_tree.clone();
        let chain = as_future
            .map(move |reports| {
                match reports {
                    Ok(report) => {
                        let as_hashmap = report.simplify();
                        println!("{}", c1.get_tree(&as_hashmap));
                    }
                    Err(report) => {
                        let as_hashmap = report.simplify();
                        println!("{}", c1.get_tree(&as_hashmap));
                    }
                };
                ()
            })
            .map_err(move |report| {
                let as_hashmap = report.simplify();
                println!("{}", c2.get_tree(&as_hashmap));
                ()
            });

        tokio::spawn(chain);

        Ok(())
    }));
}
