use thiserror::Error;

#[derive(Error, Debug)]
pub enum RobsError {
    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Encoder not found: {0}")]
    EncoderNotFound(String),

    #[error("Output not found: {0}")]
    OutputNotFound(String),

    #[error("Scene not found: {0}")]
    SceneNotFound(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Failed to create source: {0}")]
    SourceCreationFailed(String),

    #[error("Failed to create encoder: {0}")]
    EncoderCreationFailed(String),

    #[error("Failed to create output: {0}")]
    OutputCreationFailed(String),

    #[error("Failed to initialize encoder: {0}")]
    EncoderInitFailed(String),

    #[error("Failed to connect output: {0}")]
    OutputConnectFailed(String),

    #[error("Failed to encode: {0}")]
    EncodeFailed(String),

    #[error("Failed to decode: {0}")]
    DecodeFailed(String),

    #[error("Pipeline error: {0}")]
    PipelineError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Plugin error: {0}")]
    PluginError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("FFmpeg error: {0}")]
    FfmpegError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Profile error: {0}")]
    ProfileError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type RobsResult<T> = Result<T, RobsError>;

impl From<anyhow::Error> for RobsError {
    fn from(err: anyhow::Error) -> Self {
        RobsError::Unknown(err.to_string())
    }
}
