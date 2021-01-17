#[derive(Debug)]
pub struct FlamError {
    pub(crate) source: Box<dyn std::error::Error>,
}
