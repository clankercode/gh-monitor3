use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("GitHub API error: {0}")]
    GitHub(#[from] octocrab::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Window error: {0}")]
    Window(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
