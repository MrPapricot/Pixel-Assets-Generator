#[allow(dead_code)]
pub(crate) enum LogLevel {
    Info,
    Warning,
    Error,
    CriticalError,
}

pub trait Logger: Send + Sync {
    fn log(&self, message: &str, log_level: LogLevel);
}
