use crate::exec::FutureSig;
use crate::report::Report;
use crate::task::TaskItem;
use futures::future::lazy;
use futures::sync::oneshot;
use futures::Future;
use std::process::Command;
use std::process::Stdio;

pub fn task_item(task_item: TaskItem) -> FutureSig {
    let cmd_clone = task_item.cmd.clone();
    let id_clone = task_item.id.clone();
    let id_clone2 = task_item.id.clone();
    Box::new(lazy(move || {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(lazy(move || {
            let mut child = Command::new("sh");
            child.arg("-c").arg(cmd_clone);
            child.stdin(Stdio::inherit());
            child.stdout(Stdio::inherit());
            match child.status() {
                Ok(s) => {
                    if s.success() {
                        match tx.send(Ok(Report::End { id: id_clone })) {
                            Ok(s) => println!("sent oneshot OK"),
                            Err(e) => println!("failed to send sent oneshot OK={:?}", e),
                        }
                        Ok(())
                    } else {
                        match tx.send(Err(Report::Error { id: id_clone })) {
                            Ok(s) => println!("sent oneshot ERR"),
                            Err(e) => println!("failed to send sent oneshot ERR={:?}", e),
                        }
                        Err(())
                    }
                }
                Err(_e) => Err(()),
            }
        }));
        rx.map_err(move |_e| Report::Error { id: id_clone2 })
    }))
}
