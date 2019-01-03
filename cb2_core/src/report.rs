use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleReport {
    Ok { id: String, time: DateTime<Utc> },
    Err { id: String, time: DateTime<Utc> },
}

#[derive(Debug, Clone)]
pub enum Report {
    Started {
        id: String,
        time: DateTime<Utc>,
    },
    GroupStarted {
        id: String,
        time: DateTime<Utc>,
    },
    End {
        id: String,
        time: DateTime<Utc>,
        dur: chrono::Duration,
    },
    EndGroup {
        id: String,
        reports: Vec<Result<Report, Report>>,
        time: DateTime<Utc>,
        dur: chrono::Duration,
    },
    Error {
        id: String,
        time: DateTime<Utc>,
        dur: chrono::Duration,
    },
    ErrorGroup {
        id: String,
        reports: Vec<Result<Report, Report>>,
        time: DateTime<Utc>,
        dur: chrono::Duration,
    },
}
impl Report {
    pub fn flatten(&self) -> Vec<Report> {
        let mut output = Vec::new();
        collect_vec(&self, &mut output);
        output
    }
    pub fn id(&self) -> String {
        match self {
            Report::Started { id, .. }
            | Report::EndGroup { id, .. }
            | Report::GroupStarted { id, .. }
            | Report::End { id, .. }
            | Report::Error { id, .. }
            | Report::ErrorGroup { id, .. } => id.to_string(),
        }
    }
    pub fn duration_by_id(match_id: String, reports: &Vec<Report>) -> Option<f32> {
        let start = reports.iter().find_map(|report| match report {
            Report::Started { id, time, .. } => {
                if id.to_string() == match_id {
                    Some(time)
                } else {
                    None
                }
            }
            Report::GroupStarted { id, time, .. } => {
                if id.to_string() == match_id {
                    Some(time)
                } else {
                    None
                }
            }
            _ => None,
        });
        let end = reports.iter().find_map(|report| match report {
            Report::End { id, time, .. }
            | Report::EndGroup { id, time, .. }
            | Report::Error { id, time, .. }
            | Report::ErrorGroup { id, time, .. } => {
                if id.to_string() == match_id {
                    Some(time)
                } else {
                    None
                }
            }
            _ => None,
        });
        match (start, end) {
            (Some(time), Some(time_end)) => {
                println!("time={}, time_end={}", time, time_end);
                let dur = time_end.signed_duration_since(*time);
                Some((dur.num_milliseconds() as f32) / 1000 as f32)
            }
            (Some(_time), None) => {
                println!("start found, no end");
                None
            }
            (None, Some(_time)) => {
                println!("end found, no start");
                None
            }
            _ => None,
        }
    }
}

fn collect_vec(report: &Report, target: &mut Vec<Report>) {
    match report {
        Report::EndGroup {
            id: _,
            reports,
            time: _,
            dur: _,
        } => {
            target.push(report.clone());
            reports.iter().for_each(|result| match result {
                Ok(report) | Err(report) => collect_vec(&report.clone(), target),
            })
        }
        Report::ErrorGroup {
            id: _,
            reports,
            time: _,
            dur: _,
            ..
        } => {
            target.push(report.clone());
            reports.iter().for_each(|result| match result {
                Ok(report) | Err(report) => collect_vec(&report.clone(), target),
            })
        }
        Report::End { id: _, time: _, .. } => {
            target.push(report.clone());
        }
        Report::Error { id: _, time: _, .. } => {
            target.push(report.clone());
        }
        Report::Started { .. } => {
            target.push(report.clone());
        }
        Report::GroupStarted { .. } => {
            target.push(report.clone());
        }
    };
}
