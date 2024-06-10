use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;

pub fn init() -> Guards {
    let file_appender = tracing_appender::rolling::daily("./logs/", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt().with_writer(non_blocking).with_max_level(Level::DEBUG).init();

    Guards {
        worker_guard: _guard,
    }
}

pub struct Guards {
    worker_guard: WorkerGuard,
}

pub mod footstones {
    pub use tracing::{info, debug, error, warn};
}
