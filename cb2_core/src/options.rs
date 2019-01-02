use clap::App as ClapApp;
use clap::Arg;
use clap::ArgMatches;
use clap::Error;
use std::ffi::OsString;
use std::fmt;

#[derive(Debug, Default, PartialEq)]
pub struct Options {
    pub tasks: Vec<String>,
}

impl Options {
    pub fn from_args<I, T>(args: I) -> Result<Options, ProgramStartError>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = ClapApp::new("cb2")
            .arg(Arg::with_name("tasks").required(true).min_values(1))
            .get_matches_from_safe(args);

        matches
            .and_then(|matches| match matches.values_of("tasks") {
                Some(tasks_vec) => {
                    let tasks = tasks_vec.map(|s| s.to_string()).collect::<Vec<String>>();
                    Ok(Options {
                        tasks,
                        ..Default::default()
                    })
                }
                None => Ok(Options::default()),
            })
            .map_err(ProgramStartError::InvalidArgs)
        //        Ok(Options::default())
    }
}

#[derive(Debug)]
pub enum ProgramStartError {
    InvalidArgs(clap::Error),
}

impl fmt::Display for ProgramStartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            ProgramStartError::InvalidArgs(clap_error) => format!("{}", clap_error),
        };
        write!(f, "{}", output)
    }
}

#[test]
fn test_from_args() {
    let input = Options::from_args(vec!["cb2", "2", "3"]).expect("opts from vec");
    assert_eq!(
        Options {
            tasks: vec!["2".into(), "3".into()]
        },
        input
    )
}
