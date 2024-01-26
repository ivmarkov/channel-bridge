use core::convert::Infallible;

use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{DynPublisher, DynSubscriber, PubSubChannel, Publisher, Subscriber, WaitResult},
};

use super::{Receiver, Sender};

impl<'t, T> Sender for DynPublisher<'t, T>
where
    T: Clone + 't,
{
    type Error = Infallible;

    type Data = T;

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        self.publish(data.clone()).await;

        Ok(())
    }
}

impl<'t, T> Receiver for DynSubscriber<'t, T>
where
    T: Clone + 't,
{
    type Error = Infallible;

    type Data = T;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        match self.next_message().await {
            WaitResult::Message(data) => Ok(data),
            _ => panic!(),
        }
    }
}

impl<'t, M, T, const CAP: usize, const SUBS: usize, const PUBS: usize> Sender
    for Publisher<'t, M, T, CAP, SUBS, PUBS>
where
    M: RawMutex + 't,
    T: Clone + 't,
{
    type Error = Infallible;

    type Data = T;

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        self.publish(data.clone()).await;

        Ok(())
    }
}

impl<M, T, const CAP: usize, const SUBS: usize, const PUBS: usize> Sender
    for &PubSubChannel<M, T, CAP, SUBS, PUBS>
where
    M: RawMutex,
    T: Clone,
{
    type Error = Infallible;

    type Data = T;

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        self.publisher().unwrap().publish(data.clone()).await; // TODO

        Ok(())
    }
}

impl<'t, M, T, const CAP: usize, const SUBS: usize, const PUBS: usize> Receiver
    for Subscriber<'t, M, T, CAP, SUBS, PUBS>
where
    M: RawMutex + 't,
    T: Clone + 't,
{
    type Error = Infallible;

    type Data = T;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        match self.next_message().await {
            WaitResult::Message(data) => Ok(data),
            _ => panic!(),
        }
    }
}
