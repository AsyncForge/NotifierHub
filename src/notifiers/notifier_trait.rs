use super::ChannelState;
use crate::{
    closable_trait::ClosableMessage, error::NotifierError, writing_handler::WritingHandler,
};

pub trait NotifierHubTrait {
    type M;
    type ChannelId;
    type Subscribtion;

    fn subscribe(&mut self, id: &Self::ChannelId, channel_size: usize) -> Self::Subscribtion;

    fn unsubscribe(
        &mut self,
        id: &Self::ChannelId,
        subscribtion: &Self::Subscribtion,
    ) -> Result<ChannelState, NotifierError<Self::M, Self::ChannelId>>;

    fn channel_state(&self, id: &Self::ChannelId) -> ChannelState;

    fn get_channels(&self) -> Vec<Self::ChannelId>;

    fn publish(
        &self,
        msg: Self::M,
        id: &Self::ChannelId,
    ) -> Result<WritingHandler<Self::M>, NotifierError<Self::M, Self::ChannelId>>
    where
        <Self as NotifierHubTrait>::M: Send;

    fn broadcast(&self, msg: Self::M) -> WritingHandler<Self::M>
    where
        <Self as NotifierHubTrait>::M: Send;

    fn clean_channel(&mut self, channel: &Self::ChannelId) -> ChannelState;

    fn shutdown(
        &mut self,
        channel: &Self::ChannelId,
    ) -> Result<WritingHandler<Self::M>, NotifierError<Self::M, Self::ChannelId>>
    where
        <Self as NotifierHubTrait>::M: Send + ClosableMessage;
}
