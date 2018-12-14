use crate::task::RunMode;
use crate::task::Task;
use std::env;
use std::io;
use std::io::Error;
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum Report {
    Begin { id: usize },
    End { id: usize, status: ExitStatus },
    Error { id: usize, error: std::io::Error },
}

type MsgSender = Sender<Report>;
type JH = Result<JoinHandle<()>, io::Error>;

pub fn run(task: &Task) -> Vec<Report> {
    let (tx, rx) = mpsc::channel();

    process(task, &tx);

    drop(tx);

    let mut reports = Vec::new();

    for msg in rx {
        reports.push(msg);
    }

    reports
}

fn process_item(id: usize, command_input: String, tx: &MsgSender) -> JH {
    let tx1 = tx.clone();
    thread::Builder::new()
        .name(format!("thread_item,id={}", id))
        .spawn(move || {
            let mut command = Command::new("sh");

            let begin_msg = Report::Begin { id };
            tx1.send(begin_msg).expect("send begin message");

            command.arg("-c").arg(command_input);

            match command.status() {
                Ok(status) => {
                    let end_msg = Report::End { id, status };
                    tx1.send(end_msg).expect("send end message");
                    if status.success() {
                        Err(())
                    } else {
                        Ok(())
                    }
                }
                Err(error) => {
                    let begin = Report::Error { id, error };
                    tx1.send(begin).expect("send error message");
                    Err(())
                }
            };
        })
}

fn process_group(id: usize, items: Vec<Task>, run_mode: RunMode, tx: &MsgSender) -> JH {
    let tx1 = tx.clone();
    thread::Builder::new()
        .name(format!("thread_group,id={}", id))
        .spawn(move || {
            for item in items.into_iter() {
                match run_mode {
                    RunMode::Series => {
                        match process(&item, &tx1).unwrap().join() {
                            Ok(()) => { /* */ }
                            Err(e) => { /* */ }
                        };
                    }
                    RunMode::Parallel => {
                        process(&item, &tx1);
                    }
                }
            }
        })
}

fn process(task: &Task, tx: &MsgSender) -> JH {
    match task {
        Task::Item { id, command } => process_item(*id, command.to_string(), tx),
        Task::Group {
            id,
            items,
            run_mode,
        } => process_group(*id, items.to_vec(), run_mode.clone(), tx),
    }
}
