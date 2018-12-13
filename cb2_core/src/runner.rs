use crate::task::RunMode;
use crate::task::Task;
use std::env;
use std::io;
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug)]
pub enum Report {
    Begin { id: usize },
    End { id: usize },
    Error { id: usize },
}

type MsgSender = Sender<Report>;
type JH = Result<JoinHandle<()>, io::Error>;

pub fn run(task: &Task) {
    let (tx, rx) = mpsc::channel();

    process(task, &tx);

    drop(tx);

    for msg in rx {
        println!("{:?}", msg);
    }
}

fn process_item(id: usize, command_input: String, tx: &MsgSender) -> JH {
    let tx1 = tx.clone();
    thread::Builder::new()
        .name(format!("thread_item,id={}", id))
        .spawn(move || {
            let mut command = Command::new("sh");

            command.arg("-c").arg(command_input);

            match command.status() {
                Ok(status) => {
                    println!("status={}", status);
                    let begin = Report::Begin { id };
                    tx1.send(begin).unwrap();
                }
                Err(e) => {
                    println!("error={}", e);
                    let begin = Report::Error { id };
                    tx1.send(begin).unwrap();
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
