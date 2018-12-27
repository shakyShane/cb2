use crate::exec::FutureSig;
use crate::report::Report;
use crate::task::RunMode;
use crate::task::{Task, TaskGroup};
use crate::task_group::task_group;
use crate::task_item::task_item;
use futures::future::lazy;
use futures::stream::iter_ok;
use futures::sync::mpsc::Sender;
use futures::Future;
use futures::Stream;
use std::sync::{Arc, Mutex};

pub fn task_seq(group: TaskGroup, sender: Sender<Report>) -> FutureSig {
    let id_clone = group.id.clone();
    Box::new(lazy(move || {
        let track: Arc<Mutex<Vec<Result<Report, Report>>>> = Arc::new(Mutex::new(vec![]));
        let c1 = track.clone();
        let c2 = track.clone();

        let items = group.items.into_iter().map(move |item| match item {
            Task::Item(item) => task_item(item, sender.clone()),
            Task::Group(group) => match group.run_mode {
                RunMode::Series => task_seq(group, sender.clone()),
                RunMode::Parallel => task_group(group, sender.clone()),
            },
        });

        iter_ok(items)
            .for_each(move |this_future| {
                let results = c1.clone();
                this_future
                    .then(move |x| {
                        let mut next = results.lock().unwrap();
                        match x {
                            Ok(Ok(s)) => {
                                next.push(Ok(s));
                                Ok(())
                            }
                            Ok(Err(s)) => {
                                next.push(Err(s));
                                Err(())
                            }
                            Err(e) => {
                                next.push(Err(e));
                                Err(())
                            }
                        }
                    })
                    .map(|_e| ())
            })
            .then(move |_res| {
                let next = c2.clone();
                let reports = next.lock().unwrap();
                let all_valid = reports.iter().all(|x| x.is_ok());

                if all_valid {
                    Ok(Ok(Report::EndGroup {
                        id: id_clone,
                        reports: reports.clone(),
                    }))
                } else {
                    Ok(Err(Report::ErrorGroup {
                        id: id_clone,
                        reports: reports.clone(),
                    }))
                }
            })
    }))
}
