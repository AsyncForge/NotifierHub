use notifier_hub::notifier::NotifierHub;
use std::sync::Arc;
use tokio::sync::Mutex;

// This type is going to be sent among the subscribers
#[derive(Clone, Debug)]
enum Message {
    StringMessage(String),
    Number(u32),
    Close,
}

// First subscriber
fn subscriber_1(hub: Arc<Mutex<NotifierHub<Message, &'static str>>>) {
    tokio::spawn(async move {
        // Subscribing to "channel1" and "channel2"
        let mut receiver = hub
            .lock()
            .await
            .subscribe_multiple(&["channel1", "channel2"], 100);
        loop {
            let msg = receiver.recv().await.unwrap();
            match msg {
                Message::StringMessage(s_msg) => {
                    println!("Just received a new message as subscriber_1: {s_msg}")
                }
                Message::Number(n) => println!("Just received a number as subscriber_1: {n}"),
                Message::Close => break,
            }
        }
        hub.lock()
            .await
            .unsubscribe_multiple(&["channel1", "channel2"], &receiver)
            .unwrap();
    });
}

// Second subscriber
fn subscriber_2(hub: Arc<Mutex<NotifierHub<Message, &'static str>>>) {
    tokio::spawn(async move {
        // Subscribing only to "channel1"
        let mut receiver = hub.lock().await.subscribe(&"channel1", 100);
        loop {
            let msg = receiver.recv().await.unwrap();
            match msg {
                Message::StringMessage(s_msg) => {
                    println!("Just received a new message as subscriber_2: {s_msg}")
                }
                Message::Number(n) => println!("Just received a number as subscriber_2: {n}"),
                Message::Close => break,
            }
        }
        hub.lock()
            .await
            .unsubscribe(&"channel1", &receiver)
            .unwrap();
    });
}

#[tokio::main]
async fn main() {
    // Create a new NotifierHub wrapped in a mutex
    let hub = Arc::new(Mutex::new(NotifierHub::new()));

    let mut creation_waiter = hub.lock().await.get_creation_waiter(&"channel1");
    let mut destruction_waiter = hub.lock().await.get_destruction_waiter(&"channel1");

    // Start subscriber threads
    subscriber_1(hub.clone());
    subscriber_2(hub.clone());

    // Wait for subscriber_1 and subscriber_2 to be ready to receive messages on "channel1"
    creation_waiter.recv().await.unwrap();
    creation_waiter.recv().await.unwrap();

    {
        let hub = hub.lock().await;

        let msg1 = Message::StringMessage("Hello!".to_string());
        // Send the message to subscriber_1 and subscriber_2 as they are both subscribed to "channel1"
        hub.clone_send(msg1, &"channel1").unwrap();

        let msg2 = Message::Number(18);
        // Only subscriber_1 will receive this message as it is subscribed to "channel2"
        hub.clone_send(msg2, &"channel2").unwrap();

        let msg3 = Message::StringMessage("Broadcast message!".to_string());
        // Sends msg3 on all channels, so subscriber_1 will receive it twice
        hub.broadcast_clone(msg3);

        let closing_message = Message::Close;
        // Send a close message to subscriber_1 and subscriber_2
        hub.clone_send(closing_message, &"channel1").unwrap();
    }

    // Wait for both subscriber_1 and subscriber_2 to unsubscribe
    destruction_waiter.recv().await;
    destruction_waiter.recv().await;
}
