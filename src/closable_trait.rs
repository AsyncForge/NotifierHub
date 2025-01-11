/// This trait should implement message if you want to use shutdown-kind methods in the hub.
pub trait ClosableMessage {
    /// Returns the designated close message for this type.
    fn get_close_message() -> Self;
}
