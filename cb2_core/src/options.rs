use clap::App as ClapApp;
use clap::Arg;
use clap::ArgMatches;
use clap::Error;
use std::ffi::OsString;

#[derive(Debug, Default, PartialEq)]
pub struct Options {
    tasks: Vec<String>,
}

impl Options {
    pub fn from_args<I, T>(args: I) -> Result<Options, ProgramStartError>
        where
            I: IntoIterator<Item = T>,
            T: Into<OsString> + Clone,
    {
        let matches = ClapApp::new("cb2")
            .arg(Arg::with_name("tasks").required(true).min_values(1))
//            .arg(
//                Arg::with_name("proxy_timeout_secs")
//                    .short("t")
//                    .long("proxy_timeout_secs")
//                    .takes_value(true),
//            )
//            .arg(
//                Arg::with_name("port")
//                    .short("p")
//                    .long("port")
//                    .takes_value(true),
//            )
//            .arg(
//                Arg::with_name("config")
//                    .short("c")
//                    .long("config")
//                    .takes_value(true),
//            )
//            .arg(Arg::with_name("r").long("run_mode").takes_value(true))
            .get_matches_from_safe(args);

        matches
            .and_then(|matches| {
                match matches.values_of("tasks") {
                    Some(tasks_vec) => {
                        let tasks = tasks_vec
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>();
                        Ok(Options {
                            tasks,
                            ..Default::default()
                        })
                    },
                    None => Ok(Options::default()),
                }
            })
            .map_err(|e| ProgramStartError::InvalidArgs)
//        Ok(Options::default())
    }
}

#[derive(Debug)]
pub enum ProgramStartError {
    InvalidArgs
}

#[test]
fn test_from_args() {
    let input = Options::from_args(vec!["cb2", "2", "3"]).expect("opts from vec");
    assert_eq!(
        Options{
            tasks: vec!["2".into(), "3".into()]
        },
        input
    )
}
