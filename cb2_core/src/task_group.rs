use crate::exec::FutureSig;
use crate::report::Report;
use crate::task::{RunMode, Task, TaskGroup};
use crate::task_item::task_item;
use crate::task_seq::task_seq;
use chrono::Utc;
use futures::future::lazy;
use futures::sync::mpsc::Sender;
use futures::Future;
use futures::Sink;

pub fn task_group(group: TaskGroup, sender: Sender<Report>) -> FutureSig {
    let id_clone = group.id.clone();
    let begin_clone = sender.clone();
    Box::new(lazy(move || {
        let items = group.items.into_iter().map(move |item| match item {
            Task::Item(item) => task_item(item, sender.clone()),
            Task::Group(group) => match group.run_mode {
                RunMode::Series => task_seq(group, sender.clone()),
                RunMode::Parallel => task_group(group, sender.clone()),
            },
        });

        let begin_time = Utc::now();

        begin_clone
            .clone()
            .send(Report::GroupStarted {
                id: id_clone.clone(),
                time: begin_time.clone(),
            })
            .then(move |_res: _| {
                futures::collect(items).then(move |res| {
                    let (items, all_valid) = match res.clone() {
                        Ok(items) => {
                            let valid = items.iter().all(|x| x.is_ok());
                            (items, valid)
                        }
                        Err(err_report) => (vec![Err(err_report)], false),
                    };

                    if all_valid {
                        Ok(Ok(Report::EndGroup {
                            time: Utc::now(),
                            id: id_clone,
                            dur: Utc::now().signed_duration_since(begin_time),
                            reports: items.clone(),
                        }))
                    } else {
                        Ok(Err(Report::ErrorGroup {
                            time: Utc::now(),
                            id: id_clone,
                            dur: Utc::now().signed_duration_since(begin_time),
                            reports: items.clone(),
                        }))
                    }
                })
            })
    }))
}
