//! # NotifierHub Library
//!
//! `NotifierHub` is a library designed to facilitate the creation, subscription, and broadcasting of asynchronous messages through smart channels.
//! It provides utilities for error handling, custom broadcasting strategies, and notifications upon channel creation.
//!
//! # Examples without thread
//!
//! The example below demonstrates how to broadcast a message to subscribers of a channel in a single thread
//! and wait for the message to be sent using a timeout:
//!
//! ```rust
//! use notifier_hub::{notifier::NotifierHub, writing_handler::Duration};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a new NotifierHub
//!     let mut hub = NotifierHub::new();
//!
//!     // Subscribe to a channel and get a receiver
//!     let mut receiver1 = hub.subscribe(&"channel1", 100);
//!     // Subscribe to the same channel and get a receiver
//!     let mut receiver2 = hub.subscribe(&"channel1", 100);
//!
//!     // Message to broadcast
//!     let msg = "Message !";
//!
//!     // Send the message to all subscribers and get a WritingHandler to track the results
//!     let handler = hub.clone_send(msg.clone(), &"channel1").unwrap();
//!
//!     // Wait for up to 100 milliseconds for senders to put the message in the channel buffer
//!     // This ensures the message is sent successfully or times out.
//!     handler.wait(Some(Duration::from_millis(100))).await.unwrap();
//!
//!     assert_eq!(&receiver1.recv().await.unwrap(), &msg);
//!     assert_eq!(&receiver2.recv().await.unwrap(), &msg);
//! }
//! ```
//! # Examples without thread
//!
//! The example below demonstrates how to use notifier_hub to create a multithreaded notification system, where multiple subscribers listen for messages on different channels.
//!
//! ```rust
//!use notifier_hub::notifier::NotifierHub;
//!use std::sync::Arc;
//!use tokio::sync::Mutex;
//!
//! // This type is going to be sent among the subscribers
//!#[derive(Clone, Debug)]
//!enum Message {
//!    StringMessage(String),
//!    Number(u32),
//!    Close,
//!}
//!
//! // First subscriber
//!fn subscriber_1(hub: Arc<Mutex<NotifierHub<Message, &'static str>>>) {
//!    tokio::spawn(async move {
//!        // Subscribing to "channel1" and "channel2"
//!        let mut receiver = hub.lock().await.subscribe_multiple(&["channel1", "channel2"], 100);
//!        loop {
//!            let msg = receiver.recv().await.unwrap();
//!            match msg {
//!                Message::StringMessage(s_msg) => println!("Just received a new message as subscriber_1: {s_msg}"),
//!                Message::Number(n) => println!("Just received a number as subscriber_1: {n}"),
//!                Message::Close => break,
//!            }
//!        }
//!        hub.lock().await.unsubscribe_multiple(&["channel1", "channel2"], &receiver).unwrap();
//!    });
//!}
//!
//! // Second subscriber
//!fn subscriber_2(hub: Arc<Mutex<NotifierHub<Message, &'static str>>>) {
//!    tokio::spawn(async move {
//!        // Subscribing only to "channel1"
//!        let mut receiver = hub.lock().await.subscribe(&"channel1", 100);
//!        loop {
//!            let msg = receiver.recv().await.unwrap();
//!            match msg {
//!                Message::StringMessage(s_msg) => println!("Just received a new message as subscriber_2: {s_msg}"),
//!                Message::Number(n) => println!("Just received a number as subscriber_2: {n}"),
//!                Message::Close => break,
//!            }
//!        }
//!        hub.lock().await.unsubscribe(&"channel1", &receiver).unwrap();
//!    });
//!}
//!
//!#[tokio::main]
//!async fn main() {
//!    // Create a new NotifierHub wrapped in a mutex
//!    let hub = Arc::new(Mutex::new(NotifierHub::new()));
//!
//!    let mut creation_waiter = hub.lock().await.get_creation_waiter(&"channel1");
//!    let mut destruction_waiter = hub.lock().await.get_destruction_waiter(&"channel1");
//!
//!    // Start subscriber threads
//!    subscriber_1(hub.clone());
//!    subscriber_2(hub.clone());
//!
//!    // Wait for subscriber_1 and subscriber_2 to be ready to receive messages on "channel1"
//!    creation_waiter.recv().await.unwrap();
//!    creation_waiter.recv().await.unwrap();
//!
//!    {
//!        let hub = hub.lock().await;
//!
//!        let msg1 = Message::StringMessage("Hello!".to_string());
//!        // Send the message to subscriber_1 and subscriber_2 as they are both subscribed to "channel1"
//!        hub.clone_send(msg1, &"channel1").unwrap();
//!
//!        let msg2 = Message::Number(18);
//!        // Only subscriber_1 will receive this message as it is subscribed to "channel2"
//!        hub.clone_send(msg2, &"channel2").unwrap();
//!
//!        let msg3 = Message::StringMessage("Broadcast message!".to_string());
//!        // Sends msg3 on all channels, so subscriber_1 will receive it twice
//!        hub.broadcast_clone(msg3);
//!
//!        let closing_message = Message::Close;
//!        // Send a close message to subscriber_1 and subscriber_2
//!        hub.clone_send(closing_message, &"channel1").unwrap();
//!    }
//!
//!    // Wait for both subscriber_1 and subscriber_2 to unsubscribe
//!    destruction_waiter.recv().await;
//!    destruction_waiter.recv().await;
//!}
//! ```
//! ## Modules
//! This crate is organized into the following modules:

/// Contains the main `NotifierHub` structure and its associated methods.
///
/// This module defines the `NotifierHub`, which acts as the main dispatcher of channels.
/// Users can create message channels, subscribe to them, and broadcast messages to multiple subscribers.
/// It also provides features such as creation waiters that notify when a new subscription is added.
///
/// ### Key Types:
/// - `NotifierHub<M, ChannelId>`: The main structure that manages channels.
/// - `SmartChannelId`: A unique identifier for each created channel.
/// - `CreationWaiter`: A receiver that gets notified when a subscription is created.
pub mod notifier;

/// Provides the `WritingHandler` for handling broadcasts in an asynchronous context.
///
/// The `WritingHandler` is responsible for tracking the outcome of broadcasted messages.
/// It ensures that the send operation completes successfully by waiting for all messages
/// to be placed in the corresponding channel buffers.
///
/// **Important Note:**
/// - The `WritingHandler` ensures that the message is successfully sent into the channel's buffer,
///   but it does not guarantee that the message has been read by the receivers.
/// - This behavior allows for efficient and controlled message dispatching without blocking the receiver's logic.
///
/// ### Why Use `WritingHandler`?
///
/// In asynchronous systems with multiple subscribers, ensuring that messages reach their intended destinations
/// can become complex. The `WritingHandler` simplifies this by providing a mechanism to track the completion
/// of the send operation without excessive overhead or locking mechanisms.
///
pub mod writing_handler;

/// Contains definitions related to error types and handling.
///
/// This module provides all the error types used by the library, such as `NotifierError`.
/// These errors are designed to represent failures related to uninitialized channels,
/// subscription issues, and unexpected states.
///
/// ### Example
/// ```rust
/// use notifier_hub::error::NotifierError;
///
/// let error: NotifierError<String, &str> = NotifierError::ChannelUninitialized("channel1");
/// match error {
///     NotifierError::ChannelUninitialized(id) => println!("Channel '{}' is not initialized", id),
///     _ => println!("Other error occurred"),
/// }
/// ```
pub mod error;
