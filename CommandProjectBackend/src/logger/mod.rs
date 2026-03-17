pub(crate) enum LogLevel {
    Info,
    Warning,
    Error,
    CriticalError,
}

pub trait Logger: Clone + Sync + Send {
    fn log(&self, message: &str, log_level: LogLevel);
}
