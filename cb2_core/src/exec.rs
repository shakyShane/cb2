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
use crossbeam_channel;
use crossbeam_channel::bounded;
use crossbeam_channel::unbounded;
use std::cell::RefCell;
use std::thread;
use std::time::Duration;

pub type FutureSig = Box<Future<Item = Result<Report, Report>, Error = Report> + Send>;
pub type Inputs = (crossbeam_channel::Sender<String>, crossbeam_channel::Receiver<String>);

pub fn exec(
    task_tree: Task
) -> (
    impl Future<Item = Result<Report, Report>, Error = ()>,
    impl Stream<Item = Report, Error = ()>,
) {
    let (report_sender, report_receiver): (Sender<Report>, Receiver<Report>) = mpsc::channel(1_024);

    let (s, r) = unbounded();

    let executor = lazy(move || {
        let as_future = match task_tree.clone() {
            Task::Item(item) => task_item(item, report_sender.clone(), (s.clone(), r.clone())),
            Task::Group(group) => match group.run_mode {
                RunMode::Series => task_seq(group, report_sender.clone(), (s.clone(), r.clone())),
                RunMode::Parallel => task_group(group, report_sender.clone(), (s.clone(), r.clone())),
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
