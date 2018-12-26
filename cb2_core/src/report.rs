use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SimpleReport {
    Ok { id: String },
    Err { id: String },
}

#[derive(Debug, Clone)]
pub enum Report {
    Unknown,
    End {
        id: String,
    },
    EndGroup {
        id: String,
        reports: Vec<Result<Report, Report>>,
    },
    Error {
        id: String,
    },
    ErrorGroup {
        id: String,
        reports: Vec<Result<Report, Report>>,
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
        Report::ErrorGroup { id, reports, .. } | Report::EndGroup { id, reports } => {
            target.insert(id.clone(), SimpleReport::Err { id: id.to_string() });
            reports.iter().for_each(|result| match result {
                Ok(report) | Err(report) => collect(&report.clone(), target),
            })
        }
        Report::End { id } => {
            target.insert(id.clone(), SimpleReport::Ok { id: id.to_string() });
        }
        Report::Error { id } => {
            target.insert(id.clone(), SimpleReport::Err { id: id.to_string() });
        }
        Report::Unknown => unimplemented!(),
    };
}

#[test]
fn test_convert_to_hm() {
    let report_tree = Report::ErrorGroup {
        id: "cc1aa056".into(),
        reports: vec![Err(Report::ErrorGroup {
            id: "d0a35bf6".into(),
            reports: vec![Err(Report::Error {
                id: "0e5bd650".into(),
            })],
        })],
    };
    let as_hm = report_tree.simplify();
    println!("{:?}", as_hm);
}
