use std::env;
use std::fmt;
use std::io;

use crate::command_util;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Streaming timed out")]
    StreamingTimedOut,
    #[error("Tuner unavailable")]
    TunerUnavailable,
    #[error("Channel not found")]
    ChannelNotFound,
    #[error("Service not found")]
    ServiceNotFound,
    #[error("Clock not synced")]
    ClockNotSynced,
    #[error("Program not found")]
    ProgramNotFound,
    #[error("Record not found")]
    RecordNotFound,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Schedule not found")]
    ScheduleNotFound,
    #[error("Recorder not found")]
    RecorderNotFound,
    #[error("Out of range")]
    OutOfRange,
    #[error("No content")]
    NoContent,
    #[error("No logo data")]
    NoLogoData,
    #[error("Access denied")]
    AccessDenied,
    #[error("Already exists")]
    AlreadyExists,
    #[error("Program already started")]
    ProgramAlreadyStarted,
    #[error("Program will start soon")]
    ProgramWillStartSoon,
    #[error("Command failed: {0}")]
    CommandFailed(command_util::Error),
    #[error("std::fmt::error: {0}")]
    FmtError(fmt::Error),
    #[error("std::io::error: {0}")]
    IoError(io::Error),
    #[error("JSON error: {0}")]
    JsonError(serde_json::Error),
    #[error("YAML error: {0}")]
    YamlError(serde_yaml::Error),
    #[error("Querystring error: {0}")]
    QuerystringError(serde_qs::Error),
    #[error("Mustache error: {0}")]
    MustacheError(mustache::Error),
    #[error("std::env error: {0}")]
    EnvVarError(env::VarError),
    #[error("tokio::sync::broadcast error: {0:?}")]
    TokioSyncBroadcastError(tokio::sync::broadcast::error::RecvError),
    #[error("hyper error: {0:?}")]
    HyperError(hyper::Error),
    #[error("axum::http error: {0:?}")]
    AxumHttpError(axum::http::Error),
    #[error("actlet error: {0}")]
    ActletError(actlet::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

impl From<command_util::Error> for Error {
    fn from(err: command_util::Error) -> Self {
        Self::CommandFailed(err)
    }
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Self {
        Self::FmtError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::YamlError(err)
    }
}

impl From<serde_qs::Error> for Error {
    fn from(err: serde_qs::Error) -> Self {
        Self::QuerystringError(err)
    }
}

impl From<mustache::Error> for Error {
    fn from(err: mustache::Error) -> Self {
        Self::MustacheError(err)
    }
}

impl From<mustache::EncoderError> for Error {
    fn from(err: mustache::EncoderError) -> Self {
        Self::MustacheError(mustache::Error::from(err))
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Self {
        Self::EnvVarError(err)
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for Error {
    fn from(err: tokio::sync::broadcast::error::RecvError) -> Self {
        Self::TokioSyncBroadcastError(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Self::HyperError(err)
    }
}

impl From<axum::http::Error> for Error {
    fn from(err: axum::http::Error) -> Self {
        Self::AxumHttpError(err)
    }
}

impl From<actlet::Error> for Error {
    fn from(err: actlet::Error) -> Self {
        Self::ActletError(err)
    }
}
