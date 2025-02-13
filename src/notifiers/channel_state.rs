/// Represents the state of a channel. You can retrieve it by calling `channel_state` on the `NotifierHub`.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ChannelState {
    /// The initial state of the channel—no subscribers have ever connected.
    Uninitialised,
    /// The channel has active subscribers. This state remains while there is some subscriber, even if they are not active
    Running,
    /// The channel had subscribers in the past, but they have unsubscribed, or they had dropped and then clean_channel has been called
    Over,
}
