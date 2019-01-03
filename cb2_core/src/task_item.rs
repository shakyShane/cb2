use crate::exec::FutureSig;
use crate::report::Report;
use crate::task::TaskItem;
use chrono::Duration;
use chrono::Utc;
use futures::future::lazy;
use futures::sync::mpsc::Sender;
use futures::sync::oneshot;
use futures::Future;
use futures::Sink;
use std::process::Command;
use std::process::Stdio;

pub fn task_item(task_item: TaskItem, sender: Sender<Report>) -> FutureSig {
    let cmd_clone = task_item.cmd.clone();
    let id_clone = task_item.id.clone();
    let id_clone2 = task_item.id.clone();
    Box::new(lazy(move || {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(lazy(move || {
            let (tx1, rx1) = oneshot::channel();
            tokio::spawn(
                sender
                    .clone()
                    .send(Report::Started {
                        id: id_clone.clone(),
                        time: Utc::now(),
                    })
                    .then(|v| {
                        match v {
                            Ok(_x) => {}
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                        tx1.send(())
                    })
                    .map(|_val| ())
                    .map_err(|_e: ()| ()),
            );
            rx1.then(move |_report| {
                let begin_time = Utc::now();
                let mut child = Command::new("sh");
                child.arg("-c").arg(cmd_clone);
                child.stdin(Stdio::inherit());
                child.stdout(Stdio::inherit());
                match child.status() {
                    Ok(s) => {
                        let outgoing = if s.success() {
                            Report::End {
                                id: id_clone.clone(),
                                time: Utc::now(),
                                dur: Utc::now().signed_duration_since(begin_time),
                            }
                        } else {
                            Report::Error {
                                id: id_clone.clone(),
                                time: Utc::now(),
                                dur: Utc::now().signed_duration_since(begin_time),
                            }
                        };
                        tokio::spawn(
                            sender
                                .clone()
                                .send(outgoing.clone())
                                .map(|_val| ())
                                .map_err(|_e| ()),
                        );
                        match tx.send(report_wrap(outgoing)) {
                            Ok(_s) => {
                                debug!("sent oneshot for {}", id_clone);
                            }
                            Err(_e) => {
                                error!("failed to send oneshot for {}", id_clone);
                            }
                        }
                        Ok(())
                    }
                    Err(_e) => Err(()),
                }
            })
        }));
        rx.map_err(move |_e| Report::Error {
            id: id_clone2,
            time: Utc::now(),
            dur: Duration::seconds(0),
        })
    }))
}

fn report_wrap(report: Report) -> Result<Report, Report> {
    match report {
        Report::Error { .. } | Report::ErrorGroup { .. } => Err(report),
        _ => Ok(report),
    }
}
