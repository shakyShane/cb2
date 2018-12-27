use futures::future::lazy;
use futures::Future;

use crate::report::Report;
use crate::report::SimpleReport;
use crate::task::{RunMode, Task};
use crate::task_group::task_group;
use crate::task_item::task_item;
use crate::task_seq::task_seq;
use futures::sync::oneshot;
use std::collections::HashMap;
use std::sync::Arc;

pub type FutureSig = Box<Future<Item = Result<Report, Report>, Error = Report> + Send>;

pub fn exec(task_tree: Task) -> impl Future<Item = HashMap<String, SimpleReport>, Error = ()> {
    let (tx, rx) = oneshot::channel();
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

        // tokio::spawn/run need Future<Item=(),Error=()> so
        // we extract the values here and send them back out of the channel
        let chain = as_future
            .map(move |reports| {
                match reports {
                    Ok(report) => {
                        let as_hashmap = report.simplify();
                        tx.send(as_hashmap).expect("send OK reports out");
                    }
                    Err(report) => {
                        let as_hashmap = report.simplify();
                        tx.send(as_hashmap).expect("send ERR reports out");
                    }
                };
                ()
            })
            .map_err(move |report| {
                unimplemented!();
                ()
            });

        tokio::spawn(chain);

        Ok(())
    }));
    rx.map_err(|e| {
        unimplemented!();
        ()
    })
}
