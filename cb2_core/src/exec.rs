use futures::future::lazy;
use futures::Future;

use crate::report::Report;
use crate::task::{RunMode, Task};
use crate::task_group::task_group;
use crate::task_item::task_item;
use crate::task_seq::task_seq;

pub type FutureSig = Box<Future<Item = Result<Report, Report>, Error = ()> + Send>;

pub fn exec(task_tree: Task) {
    tokio::run(lazy(move || {
        let as_future = match task_tree {
            Task::Item(item) => task_item(item),
            Task::Group(group) => match group.run_mode {
                RunMode::Series => task_seq(group),
                RunMode::Parallel => task_group(group),
            },
        };

        let chain = as_future
            .map(|val| {
                println!("outgoing = {:?}", val);
                ()
            })
            .map_err(|e| {
                println!("Err made it to the top = {:?}", e);
                ()
            });

        tokio::spawn(chain);
        //        tokio::spawn(chain.map(|x| ()).map_err(|e| ()));

        //        let output = vec![
        //            item("echo before && sleep 1 && echo after"),
        //            item("echos 1"),
        //            item("echo 2"),
        //        ].iter().map(stream::iter_ok).flatten_stream();

        Ok(())
    }));
}
