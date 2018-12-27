use crate::exec::FutureSig;
use crate::report::Report;
use crate::task::TaskItem;
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
            tokio::spawn(
                sender
                    .clone()
                    .send(Report::Begin {
                        id: id_clone.clone(),
                    })
                    .map(|val| ())
                    .map_err(|e| ()),
            );
            let mut child = Command::new("sh");
            child.arg("-c").arg(cmd_clone);
            child.stdin(Stdio::inherit());
            child.stdout(Stdio::inherit());
            match child.status() {
                Ok(s) => {
                    let outgoing = if s.success() {
                        Report::End {
                            id: id_clone.clone(),
                        }
                    } else {
                        Report::Error {
                            id: id_clone.clone(),
                        }
                    };
                    tokio::spawn(
                        sender
                            .clone()
                            .send(outgoing.clone())
                            .map(|val| ())
                            .map_err(|e| ()),
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
        }));
        rx.map_err(move |_e| Report::Error { id: id_clone2 })
    }))
}

fn report_wrap(report: Report) -> Result<Report, Report> {
    match report {
        Report::Error { .. } | Report::ErrorGroup { .. } => Err(report),
        _ => Ok(report),
    }
}
