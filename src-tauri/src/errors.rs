use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppError {
    #[error("Server unreachable: {message}")]
    ServerUnreachable { message: String },
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
    #[error("State error: {message}")]
    State { message: String },
    #[error("Config storage error: {message}")]
    ConfigStorage { message: String },
}
