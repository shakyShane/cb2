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
