use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Default)]
pub struct FilterState {
    pub sources: Vec<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}
