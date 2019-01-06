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
use crossbeam_channel;
use std::sync::Arc;
use shared_child::SharedChild;
use std::sync::Mutex;
use crate::exec::Inputs;

pub fn task_item(task_item: TaskItem, sender: Sender<Report>, inputs: Inputs) -> FutureSig {
    let cmd_clone = task_item.cmd.clone();
    let id_clone = task_item.id.clone();
    let id_clone2 = task_item.id.clone();
    let (task_complete_msg, task_complete_recv) = oneshot::channel();
    let (kill_sender, _) = inputs.clone();

    if cmd_clone == "KILL".to_string() {
        return Box::new(lazy(move || {
            let killed_report = Report::End {
                id: id_clone.clone(),
                time: Utc::now(),
                dur: Duration::seconds(0),
            };
            match task_complete_msg.send(report_wrap(killed_report)) {
                Ok(_s) => {
                    debug!("sent oneshot for {}", id_clone);
                }
                Err(_e) => {
                    error!("failed to send oneshot for {}", id_clone);
                }
            }
            task_complete_recv
                .inspect(move |val| {
                    kill_sender.send("KILL".to_string());
                    ()
                })
                .map_err(move |_e| {
                Report::Error {
                    id: id_clone2,
                    time: Utc::now(),
                    dur: Duration::seconds(0),
                }
            })
        }));
    }
    Box::new(lazy(move || {
        tokio::spawn(lazy(move || {
            let (begin_msg_tsx, begin_msg_rx) = oneshot::channel();
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
                        begin_msg_tsx.send(())
                    })
                    .map(|_val| ())
                    .map_err(|_e: ()| ()),
            );
            begin_msg_rx.then(move |_report| {

                let begin_time = Utc::now();
                let mut child_process = Command::new("sh");
                child_process.arg("-c").arg(cmd_clone);
                child_process.stdin(Stdio::inherit());
                child_process.stdout(Stdio::inherit());

                let shared_child = SharedChild::spawn(&mut child_process).expect("wrapped");
                let child_arc = Arc::new(shared_child);

                let (_, kill_recv) = inputs;
                let child_clone = child_arc.clone();
                let was_killed = Arc::new(Mutex::new(false));

                let was_killed_trigger = was_killed.clone();
                let was_killed_clone = was_killed.clone();
                let id_clone_2 = id_clone.clone();

                tokio::spawn(lazy(move || {
                    match kill_recv.recv() {
                        Ok(s) => {
                            if s == "KILL" {
                                println!("received kill-shot {}", id_clone_2);
                                *was_killed_trigger.lock().unwrap() = true;
                                child_clone.kill().expect("KILLLED");
                            } else {
                                println!("GOT SOMETHING ELSE = {}", s);
                            }
                        },
                        Err(e) => println!("inputs.recv.recv()-Err={}", e),
                    }
                    Ok(())
                }));

                match child_arc.wait() {
                    Ok(s) => {
                        let outgoing_report = if s.success() {
                            Report::End {
                                id: id_clone.clone(),
                                time: Utc::now(),
                                dur: Utc::now().signed_duration_since(begin_time),
                            }
                        } else {
                            println!("WAS IT KILLED?? = {:?}", *was_killed_clone.lock().unwrap());
                            Report::Error {
                                id: id_clone.clone(),
                                time: Utc::now(),
                                dur: Utc::now().signed_duration_since(begin_time),
                            }
                        };

                        let report_clone = outgoing_report.clone();
                        let (kill_sender, _) = inputs;
                        tokio::spawn(lazy(move || {
                            sender
                                .clone()
                                .send(report_clone)
                                .inspect(move |output| {
                                    kill_sender.send("DONE".to_string());
                                    ()
                                })
                                .map(|_val| ())
                                .map_err(|_e| ())
                        }));


                        match task_complete_msg.send(report_wrap(outgoing_report.clone())) {
                            Ok(_s) => {
                                debug!("sent oneshot for {}", id_clone);
                            }
                            Err(_e) => {
                                error!("failed to send oneshot for {}", id_clone);
                            }
                        }
                        Ok(())
                    }
                    Err(_e) => unimplemented!(),
                }
            })
        }));
        task_complete_recv.map_err(move |_e| {
            Report::Error {
                id: id_clone2,
                time: Utc::now(),
                dur: Duration::seconds(0),
            }
        })
    }))
}

fn report_wrap(report: Report) -> Result<Report, Report> {
    match report {
        Report::Error { .. } | Report::ErrorGroup { .. } => Err(report),
        _ => Ok(report),
    }
}
