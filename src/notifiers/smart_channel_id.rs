/// `SmartChannelId` is a unique identifier for channels within a `NotifierHub`.
/// It consists of a monotonically increasing counter and the memory address of the `NotifierHub`
/// (converted to `usize`). This guarantees that the ID is unique across different contexts.
///
/// The address represents a specific field of a specific `NotifierHub`, ensuring its global uniqueness.
/// We store the address as a `usize` instead of a raw pointer to simplify the type and to keep this type simple without involving generics.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct SmartChannelId {
    /// A counter that increments with each created channel to ensure uniqueness.
    pub(crate) channel_counter: usize,
    /// The memory address of the `NotifierHub`, stored as a `usize` for simplicity (used as an identifier, not as a dereferenceable address).
    pub(crate) notifier_address: usize,
}
