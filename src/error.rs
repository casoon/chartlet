use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartError {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl ChartError {
    pub(crate) fn new(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ChartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for ChartError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartWarning {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

impl ChartWarning {
    pub(crate) fn new(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
}
