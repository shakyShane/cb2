use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleReport {
    Ok { id: String, time: DateTime<Utc> },
    Err { id: String, time: DateTime<Utc> },
}

#[derive(Debug, Clone)]
pub enum Report {
    Unknown,
    Begin {
        id: String,
        time: DateTime<Utc>,
    },
    End {
        id: String,
        time: DateTime<Utc>,
    },
    EndGroup {
        id: String,
        reports: Vec<Result<Report, Report>>,
        time: DateTime<Utc>,
    },
    Error {
        id: String,
        time: DateTime<Utc>,
    },
    ErrorGroup {
        id: String,
        reports: Vec<Result<Report, Report>>,
        time: DateTime<Utc>,
    },
}
impl Report {
    pub fn simplify(self) -> HashMap<String, SimpleReport> {
        let mut output = HashMap::new();
        collect(&self, &mut output);
        output
    }
}

fn collect(report: &Report, target: &mut HashMap<String, SimpleReport>) {
    match report {
        Report::EndGroup { id, reports, time } => {
            target.insert(
                id.clone(),
                SimpleReport::Ok {
                    id: id.to_string(),
                    time: time.clone(),
                },
            );
            reports.iter().for_each(|result| match result {
                Ok(report) | Err(report) => collect(&report.clone(), target),
            })
        }
        Report::ErrorGroup {
            id, reports, time, ..
        } => {
            target.insert(
                id.clone(),
                SimpleReport::Err {
                    id: id.to_string(),
                    time: time.clone(),
                },
            );
            reports.iter().for_each(|result| match result {
                Ok(report) | Err(report) => collect(&report.clone(), target),
            })
        }
        Report::End { id, time } => {
            target.insert(
                id.clone(),
                SimpleReport::Ok {
                    id: id.to_string(),
                    time: time.clone(),
                },
            );
        }
        Report::Error { id, time } => {
            target.insert(
                id.clone(),
                SimpleReport::Err {
                    id: id.to_string(),
                    time: time.clone(),
                },
            );
        }
        Report::Unknown => unimplemented!(),
        Report::Begin { .. } => { /* noop */ }
    };
}

#[test]
fn test_convert_to_hm_errors() {
    let report_tree = Report::ErrorGroup {
        time: Utc::now(),
        id: "cc1aa056".into(),
        reports: vec![Err(Report::ErrorGroup {
            time: Utc::now(),
            id: "d0a35bf6".into(),
            reports: vec![Err(Report::Error {
                id: "0e5bd650".into(),
            })],
        })],
    };
    let as_hm = report_tree.simplify();
    println!("{:?}", as_hm);
}

#[test]
fn test_convert_to_hm_ok() {
    let report_tree = Report::EndGroup {
        time: Utc::now(),
        id: "cc1aa056".into(),
        reports: vec![Ok(Report::EndGroup {
            time: Utc::now(),
            id: "d0a35bf6".into(),
            reports: vec![Ok(Report::End {
                time: Utc::now(),
                id: "0e5bd650".into(),
            })],
        })],
    };
    let as_hm = report_tree.simplify();
    let expected = [
        (
            "cc1aa056".to_string(),
            SimpleReport::Ok {
                id: "cc1aa056".to_string(),
            },
        ),
        (
            "d0a35bf6".to_string(),
            SimpleReport::Ok {
                id: "d0a35bf6".to_string(),
            },
        ),
        (
            "0e5bd650".to_string(),
            SimpleReport::Ok {
                id: "0e5bd650".to_string(),
            },
        ),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<String, SimpleReport>>();
    assert_eq!(as_hm, expected);
}
