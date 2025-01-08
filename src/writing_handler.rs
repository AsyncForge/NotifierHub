use std::sync::Arc;
pub use tokio::time::Duration;
use tokio::{sync::mpsc::error::SendError, task::JoinHandle, time::timeout};

use crate::{
    error::{NotifierError, UnexpectedErrorKind},
    notifier::{Sender, SmartChannelId},
};

type Handler<M> = JoinHandle<Result<(), SendError<M>>>;

/// `WritingHandler` is responsible for managing and awaiting multiple asynchronous
/// tasks that send messages via Tokio channels.
/// It allows for broadcasting messages to multiple senders and waiting for all tasks to complete.
/// You can either broadcast a message behind an `Arc` (for efficiency) or clone the message.
#[derive(Default)]
pub struct WritingHandler<M: Send + 'static> {
    handlers: Vec<Handler<M>>,
}

fn get_handler<M: Send + 'static>(sender: Sender<M, SmartChannelId>, msg: M) -> Handler<M> {
    tokio::spawn(async move { sender.send(msg).await })
}

impl<M: Send + 'static + Sync> WritingHandler<Arc<M>> {
    /// Creates a `WritingHandler` for broadcasting messages across multiple senders using `Arc<M>`.
    /// This avoids cloning the message for each sender but requires `M` to implement `Sync`.
    /// This approach is efficient for large messages.
    pub(crate) fn new_arc_broadcast(msg: M, senders: &[Sender<Arc<M>, SmartChannelId>]) -> Self {
        let msg = Arc::new(msg);
        WritingHandler {
            handlers: senders
                .iter()
                .map(|sender| {
                    let msg = Arc::clone(&msg);
                    let sender = sender.clone();
                    get_handler(sender, msg)
                })
                .collect(),
        }
    }
}
impl<M: Send + 'static + Clone> WritingHandler<M> {
    /// Creates a `WritingHandler` by cloning the message for each sender.
    /// This is useful when sending simple notification messages.
    pub(crate) fn new_cloning_broadcast(msg: &M, senders: &[Sender<M, SmartChannelId>]) -> Self {
        WritingHandler {
            handlers: senders
                .iter()
                .map(|sender| {
                    let msg = msg.clone();
                    let sender = sender.clone();
                    get_handler(sender, msg)
                })
                .collect(),
        }
    }
}

impl<M: Send + 'static> WritingHandler<M> {
    /// Returns an empty handler with no tasks.
    /// Calling `wait` on this handler returns immediately with success.
    pub fn empty() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Returns the number of writing.
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Returns true if the handler is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Waits for all tasks in the handler to finish.
    /// If `duration` is `None`, this method waits indefinitely.
    /// If `duration` is `Some`, it waits only for the given time.
    /// Returns the number of completed tasks on success or a vector of caught errors.
    /// Note that here the second generic type is unit as we are not using it anyway in the returned errors.
    pub async fn wait(self, duration: Option<Duration>) -> Result<usize, NotifierError<M, ()>> {
        let n = self.handlers.len();
        let mut errors = Vec::new();

        for handler in self.handlers {
            let result = match duration {
                Some(duration) => timeout(duration, handler).await,
                None => Ok(handler.await),
            };

            match result {
                Ok(Ok(Ok(()))) => (),
                Ok(Ok(Err(e))) => errors.push(NotifierError::SendingError(e)),
                Ok(Err(e)) => errors.push(NotifierError::JoiningError(e)),
                Err(_) => errors.push(NotifierError::WritingTimeout(match duration {
                    Some(d) => d,
                    None => {
                        return Err(NotifierError::UnexpectedError(
                            UnexpectedErrorKind::DurationIsMissing, // Should never append as if duration is None we put the result in Ok
                        ));
                    }
                })),
            }
        }

        if errors.is_empty() {
            Ok(n)
        } else {
            Err(NotifierError::WritingSendError(errors))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smart_channel::channel;

    const TEST_ID: SmartChannelId = SmartChannelId {
        channel_counter: 1,
        notifier_address: 1,
    };

    #[tokio::test]
    async fn test_empty_handler_wait() {
        let handler: WritingHandler<String> = WritingHandler::empty();
        let result = handler.wait(None).await;
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_handler_len() {
        let handler: WritingHandler<String> = WritingHandler::empty();
        assert_eq!(handler.len(), 0);
        assert!(handler.is_empty());
        let (tx1, _) = channel(10, TEST_ID);
        let (tx2, _) = channel(10, TEST_ID);

        let message = "Hello from Arc!";
        let handler = WritingHandler::new_arc_broadcast(message, &[tx1, tx2]);
        assert!(handler.len() == 2)
    }

    #[tokio::test]
    async fn test_arc_broadcast_success() {
        let (tx1, mut rx1) = channel(10, TEST_ID);
        let (tx2, mut rx2) = channel(10, TEST_ID);

        let message = "Hello from Arc!";
        let handler = WritingHandler::new_arc_broadcast(message, &[tx1, tx2]);
        handler.wait(None).await.unwrap();

        assert_eq!(rx1.recv().await.unwrap(), Arc::new("Hello from Arc!"));
        assert_eq!(rx2.recv().await.unwrap(), Arc::new("Hello from Arc!"));
    }

    #[tokio::test]
    async fn test_cloning_broadcast_success() {
        let (tx1, mut rx1) = channel(10, TEST_ID);
        let (tx2, mut rx2) = channel(10, TEST_ID);

        let message = "Hello from Arc!".to_string();
        let handler = WritingHandler::new_cloning_broadcast(&message, &[tx1, tx2]);
        handler.wait(None).await.unwrap();

        assert_eq!(rx1.recv().await.unwrap(), String::from("Hello from Arc!"));
        assert_eq!(rx2.recv().await.unwrap(), String::from("Hello from Arc!"));
    }

    #[tokio::test]
    async fn test_timeout_error() {
        let (tx1, _rx1) = channel(1, TEST_ID);

        let valid_handler = WritingHandler::new_cloning_broadcast(
            &"Message should pass".to_string(),
            &[tx1.clone()],
        );
        valid_handler.wait(None).await.unwrap();

        let err_handler = WritingHandler::new_cloning_broadcast(
            &"Message should not pass".to_string(),
            &[tx1.clone()],
        ); // The channel is full because of the previous messages, but the receiver never read so the sending is infinite

        let result = err_handler.wait(Some(Duration::from_millis(500))).await;
        assert!(result.is_err());

        if let Err(NotifierError::WritingSendError(errors)) = result {
            assert!(errors.len() == 1);
            assert!(matches!(errors[0], NotifierError::WritingTimeout(_)));
        } else {
            panic!("Expected timeout error.");
        }
    }

    #[tokio::test]
    async fn test_send_error() {
        let (tx, _) = channel(10, TEST_ID); // Receiver dropped intentionally.

        let handler = WritingHandler::new_cloning_broadcast(&"Join test".to_string(), &[tx]);

        let result = handler.wait(None).await;
        assert!(result.is_err());
        if let Err(NotifierError::WritingSendError(errors)) = result {
            assert!(matches!(errors[0], NotifierError::SendingError(_)));
        } else {
            panic!("Expected join error.");
        }
    }

    #[tokio::test]
    async fn test_multiple_errors() {
        let (tx1, _) = channel(10, TEST_ID); // Dropped receiver.
        let (tx2, _) = channel(10, TEST_ID); // Dropped receiver.

        let handler =
            WritingHandler::new_cloning_broadcast(&"Multi-error test".to_string(), &[tx1, tx2]);

        let result = handler.wait(None).await;
        assert!(result.is_err());
        if let Err(NotifierError::WritingSendError(errors)) = result {
            assert_eq!(errors.len(), 2); // Two sends should fail
        } else {
            panic!("Expected multiple send errors.");
        }
    }

    #[tokio::test]
    async fn test_no_error_with_successful_senders() {
        let (tx, mut rx) = channel(10, TEST_ID);

        let handler = WritingHandler::new_cloning_broadcast(&"Success message".to_string(), &[tx]);
        tokio::spawn(async move {
            let _ = rx.recv().await;
        });

        let result = handler.wait(None).await;
        assert!(result.is_ok());
    }
}
