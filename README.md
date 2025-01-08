# **AsyncForge**

**AsyncForge** is a powerful and modular toolkit for managing asynchronous tasks in **Rust**, built on top of **Tokio**. It provides a suite of crates that simplify asynchronous programming through customizable tools, procedural macros, and robust communication mechanisms.

---

## **Crate Overview**

### 1. **Task Forge** (`task_forge`)

A **flexible and performant task pool** for managing asynchronous tasks. This crate allows you to spawn multiple tasks, send messages to them, and listen for their output.

**Key Features**:
- Efficiently spawn and manage concurrent tasks.
- Send and receive custom messages to tasks using channels.
- Notifications for task creation, output, and termination.

**Example**:
```rust
use task_forge::{TaskPool, TaskTrait};
use tokio::sync::mpsc::Sender;

struct EchoTask;

impl TaskTrait<String, String, String> for EchoTask {
    fn begin(_: String, task_interface: task_forge::TaskInterface<String>) -> Sender<String> {
        let (sender, mut receiver) = task_forge::channel(10);
        tokio::spawn(async move {
            if let Some(input) = receiver.recv().await {
                let response = format!("Echo: {input}");
                task_interface.output(response).await.unwrap();
            }
            });
        sender
    }
}

#[tokio::main]
async fn main() {
    let (pool, _) = TaskPool::new();
    pool.new_task::<EchoTask, _>(1, "Hello World!".to_string()).await.unwrap();
    pool.send(1, "Ping!".to_string()).await.unwrap();

    let mut results = pool.new_result_redirection().await;
    if let Some(result) = results.recv().await {
        println!("Task Output: {}", result.output);
    }
}
```
--- 

### 2. **NotifierHub** (`notifier_hub`)

**A notification system** for broadcasting messages to multiple subscribers asynchronously. This crate is designed for scenarios where you need to notify many subscribers of events without blocking the main execution flow.

**Key Features**

- **Asynchronous notifications**: Broadcast messages to multiple subscribers using asynchronous tasks without blocking.
- **Subscription management**: Easily subscribe and unsubscribe receivers from specific channels.
- **Wait-for-send guarantees**: Use the `WritingHandler` to confirm that messages are successfully sent with flexible timeout
- **Channel creation waiters**: Provide the possibility to wait for the creation of a specific channel
- **Efficient data handling**: Supports sending Arc<M> messages to avoid unnecessary data cloning for large payloads. 

**Exemple**

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

### 3. **Smart Channel** (`smart_channel`)

An **enhanced asynchronous communication system** that provides channels with additional functionalities, such as sender identification.

**Key Features**:
- **ID-Linked Channels**: Each sender and receiver is associated with a unique connection ID, enabling verification of sender-receiver bindings.
- **Lightweight Wrappers**: Sender and receiver types wrap Tokio's standard channels, adding extra metadata while keeping the same behavior.
- **Interoperability**: SmartChannel supports direct dereferencing to use all standard Tokio sender/receiver methods.
- **Custom Bindings**: Provides bind and channel functions to associate custom IDs with channels.

```rust
use smart_channel::channel;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = channel(5, 1);

    tokio::spawn(async move {
        sender.send("Hello from sender 1!").await.unwrap();
    });

    if let Some(message) = receiver.recv().await {
        println!("Received: {}", message);
    }
}
```

## **Contributing**

We welcome contributions! Please open an issue or submit a pull request on GitHub.
