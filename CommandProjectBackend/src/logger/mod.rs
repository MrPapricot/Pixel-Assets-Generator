pub(crate) enum LogLevel {
    Info,
    Warning,
    Error,
    CriticalError
}

pub trait Logger: Clone {
    fn log(&self, message: &str, log_level: LogLevel);
}