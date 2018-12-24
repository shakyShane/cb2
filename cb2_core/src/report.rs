#[derive(Debug, Clone)]
pub enum Report {
    Unknown,
    End {
        id: usize,
    },
    EndGroup {
        id: usize,
        reports: Vec<Result<Report, Report>>,
    },
    Error {
        id: usize,
    },
    ErrorGroup {
        id: usize,
        reports: Vec<Result<Report, Report>>,
    },
}
