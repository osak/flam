use chrono::Utc;
use chrono::DateTime;

#[derive(Debug)]
pub struct Entry<T> {
    pub(crate) source: &'static str,
    pub(crate) ref_id: String,
    pub(crate) created: DateTime<Utc>,
    pub(crate) last_update: DateTime<Utc>,
    pub(crate) data: T
}