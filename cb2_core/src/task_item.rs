use crate::exec::FutureSig;
use crate::report::Report;
use crate::task::TaskItem;
use futures::future::lazy;
use futures::sync::oneshot;
use futures::Future;
use std::process::Command;

pub fn task_item(task_item: TaskItem) -> FutureSig {
    let cmd_clone = task_item.cmd.clone();
    let id_clone = task_item.id.clone();
    Box::new(lazy(move || {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(lazy(move || {
            let mut child = Command::new("sh");
            child.arg("-c").arg(cmd_clone);
            match child.status() {
                Ok(s) => {
                    if s.success() {
                        tx.send(Ok(Report::End { id: id_clone }))
                            .expect("should sent one-shot Ok");
                        Ok(())
                    } else {
                        tx.send(Err(Report::Error { id: id_clone }))
                            .expect("should sent one-shot Err");
                        Err(())
                    }
                }
                Err(_e) => Err(()),
            }
        }));
        rx.map_err(|_e| ())
    }))
}
