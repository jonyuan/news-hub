use chrono::{DateTime, Utc};
use std::time::Duration;

/// Severity level determines styling and persistence behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
    Loading,
}

/// A status message to display in the status bar
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub level: MessageLevel,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub auto_dismiss_after: Option<Duration>,
}

impl StatusMessage {
    pub fn info(text: String) -> Self {
        Self {
            level: MessageLevel::Info,
            text,
            timestamp: Utc::now(),
            auto_dismiss_after: Some(Duration::from_secs(5)),
        }
    }

    pub fn success(text: String) -> Self {
        Self {
            level: MessageLevel::Success,
            text,
            timestamp: Utc::now(),
            auto_dismiss_after: Some(Duration::from_secs(3)),
        }
    }

    pub fn warning(text: String) -> Self {
        Self {
            level: MessageLevel::Warning,
            text,
            timestamp: Utc::now(),
            auto_dismiss_after: Some(Duration::from_secs(5)),
        }
    }

    pub fn error(text: String) -> Self {
        Self {
            level: MessageLevel::Error,
            text,
            timestamp: Utc::now(),
            auto_dismiss_after: None, // Errors persist
        }
    }

    pub fn loading(text: String) -> Self {
        Self {
            level: MessageLevel::Loading,
            text,
            timestamp: Utc::now(),
            auto_dismiss_after: None,
        }
    }

    /// Check if message should be auto-dismissed based on age
    pub fn should_dismiss(&self) -> bool {
        if let Some(duration) = self.auto_dismiss_after {
            let age = Utc::now()
                .signed_duration_since(self.timestamp)
                .to_std()
                .unwrap_or(Duration::ZERO);
            age >= duration
        } else {
            false
        }
    }
}
