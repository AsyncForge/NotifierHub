# NotifierHub

A **Rust crate** providing an efficient and asynchronous notification system for broadcasting messages to multiple subscribers. **NotifierHub** is designed for scenarios that require high performance and concurrent message broadcasting, ensuring non-blocking sends and the ability to wait for completion with timeouts if desired.

## **Features**

- **Asynchronous notifications**: Broadcast messages to multiple subscribers using asynchronous tasks.
- **Subscription management**: Easily subscribe and unsubscribe receivers from specific channels.
- **Wait-for-send guarantees**: Use the `WritingHandler` to confirm that messages are successfully sent with flexible timeout
- **Channel creation waiters**: Provide the possibility to wait for the creation of a given channel
- **Efficient data handling**: Supports sending Arc<M> messages to avoid unnecessary data cloning for large payloads. 

---

## **Installation**

To use Notifier Hub, add the following to your `Cargo.toml`:
```toml
[dependencies]
notifier_hub = "0.1.0"
```

---

## **Getting Started**

Here’s a quick example demonstrating how to use **NotifierHub** to broadcast messages and wait for completion:

### **Exemple**

```rust
use notifier_hub::{notifier::NotifierHub, writing_handler::Duration};
    
#[tokio::main]
async fn main() {
    // Create a new NotifierHub
    let mut hub = NotifierHub::new();
    
    // Subscribe to a channel and get a receiver
    let mut receiver1 = hub.subscribe(&"channel1");
    // Subscribe to the same channel and get a receiver
    let mut receiver2 = hub.subscribe(&"channel1");

    // Message to broadcast
    let msg = "Message !";

    // Send the message to all subscribers and get a WritingHandler to track the results
    let handler = hub.clone_send(&msg, &"channel1").unwrap();

    // Wait for up to 100 milliseconds for senders to put the message in the channel buffer
    // This ensures the message is sent successfully or times out.
    handler.wait(Some(Duration::from_millis(100))).await.unwrap();

    assert_eq!(&receiver1.recv().await.unwrap(), &msg);
    assert_eq!(&receiver2.recv().await.unwrap(), &msg);
}
```

---

## **Key Concepts**
### NotifierHub

`NotifierHub` is the core structure that manages message broadcasting. It holds multiple channels, each identified by an ID, and allows subscribers to register for notifications and also provides some creation waiters able to wait for the creation of a specific channel.

### WritingHandler

`WritingHandler` tracks the status of broadcasted messages. It provides methods to wait for all sends to complete and set optional timeout to avoid indefinite waiting.

### Message Types

You can send messages in two ways. You can use the `clone_send` on NotifierHub that will broadcast the given message to all the subscribers by cloning it. Or the arc_send method using a shared reference to broadcast the data among the subscribers without cloning it.

---

## **API Overview**

### Creating a `NotifierHub`
```rust
let mut hub = NotifierHub::new();
```
### subscribing to channels

```rust
let receiver = hub.subscribe(&"channel_id");
```

### Broadcasting Messages through a single channel
**Using the cloning broadcast**:
```rust
let message = "Test message".to_string();
let handler = hub.clone_send(&message, &"channel_id").unwrap();
handler.wait(None).await.unwrap(); // Not necessary, waits for the message to be put in the channel
```
**Using Arc Broadcast (for large messages)**:
```rust
let large_msg = vec![0u8; 10_000_000]; // Large data
let handler = hub.arc_send(large_msg, &"channel_id"); // Will wrap it into an Arc and share it
handler.wait(None).await.unwrap(); // Not necessary, waits for the message to be put in the channel
```
### Unsubscribing
```rust
hub.unsubscribe(&"channel_id", &receiver).unwrap();
```

### Getting the State of a Channel
```rust
use notifier_hub::notifier::ChannelState;
match hub.channel_state(&"channel_id") {
    ChannelState::Running => println!("Channel is active"),
    ChannelState::Over => println!("Channel has ended"),
    ChannelState::Uninitialised => println!("Channel is uninitialised"),
}
```

### Creation Waiters
You can register waiters to be notified when new subscribers join a channel:
```rust
let mut creation_waiter = hub.get_creation_waiter(&"channel1");
let _receiver = hub.subscribe(&"channel1");
assert!(creation_waiter.recv().await.is_some());
```
### Subscribe to Multiple Channels
```rust
let receiver = hub.subscribe_multiple(&["channel1", "channel2"]);
```

## **Contributing**

We welcome contributions! Please open an issue or submit a pull request on GitHub.

## **License**

This project is licensed under the MIT License. See the LICENSE file for details.
