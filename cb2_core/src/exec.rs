use futures::future::lazy;
use futures::Future;

use crate::report::Report;
use crate::task::{RunMode, Task};
use crate::task_group::task_group;
use crate::task_item::task_item;
use crate::task_seq::task_seq;
use futures::sync::mpsc;
use futures::sync::mpsc::{Receiver, Sender};
use futures::Stream;

pub type FutureSig = Box<Future<Item = Result<Report, Report>, Error = Report> + Send>;

pub fn exec(
    task_tree: Task,
) -> (
    impl Future<Item = Result<Report, Report>, Error = ()>,
    impl Stream<Item = Report, Error = ()>,
) {
    let (report_sender, report_receiver): (Sender<Report>, Receiver<Report>) = mpsc::channel(1_024);

    let executor = lazy(move || {
        let as_future = match task_tree.clone() {
            Task::Item(item) => task_item(item, report_sender.clone()),
            Task::Group(group) => match group.run_mode {
                RunMode::Series => task_seq(group, report_sender.clone()),
                RunMode::Parallel => task_group(group, report_sender.clone()),
            },
        };

        // tokio::spawn/run need Future<Item=(),Error=()> so
        // we extract the values here and send them back out of the channel
        as_future.map_err(move |_report| {
            unimplemented!();
        })
    });

    (executor, report_receiver)
}
