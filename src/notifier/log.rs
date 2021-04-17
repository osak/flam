use log::info;

pub struct LoggingNotifier;

impl LoggingNotifier {
    pub fn new() -> Self {
        Self {}
    }

    pub fn notify(&self, headline: &str, body: &str) {
        info!("{}: {}", headline, body);
    }
}
