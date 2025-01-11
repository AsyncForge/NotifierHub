use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError, time::Duration};

#[macro_export]
macro_rules! unexpected {
    ($kind:ident) => {
        return Err(NotifierError::UnexpectedError(UnexpectedErrorKind::$kind))
    };
}

#[derive(Debug)]
pub enum UnexpectedErrorKind {
    DurationIsMissing,
    InvalidChannelStateUnsubscribe,
    SenderIsMissing,
}

#[derive(Debug, Error)]
pub enum NotifierError<M, ChannelId> {
    #[error("Failed to send a message via tokio channel beacause of this: {0:?}")]
    SendingError(SendError<M>),
    #[error("Failed to wait for a writing because of this: {0:?}")]
    JoiningError(JoinError),
    /// This one returns a vector conaining all the send errors and join errors during the writing phase
    #[error("Failed to send a message from the writing handler due to this: {0:?}")]
    WritingSendError(Vec<NotifierError<M, ChannelId>>),
    #[error("Timeout during the wait of a writing task, duration: {0:?}")]
    WritingTimeout(Duration),
    #[error("This error was not expected. Please report an issue to https://github.com/ZivoMartin/AsyncForge with this code: {0:?}")]
    UnexpectedError(UnexpectedErrorKind),
    #[error("The given receiver is no subscribed to the channel {0:?}")]
    NotSubscribed(ChannelId),
    #[error("The given receiver is no subscribed to this channels: {0:?}")]
    NotSubscribedMultiple(Vec<NotifierError<M, ChannelId>>),
    #[error("The channel {0:?} has not been initialized")]
    ChannelUninitialized(ChannelId),
    #[error("The channel {0:?} is over")]
    ChannelOver(ChannelId),
    #[error("The channel {0:?} does not exist")]
    ChannelNotExist(ChannelId),
}
