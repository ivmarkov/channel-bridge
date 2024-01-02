use core::convert::Infallible;

use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{DynPublisher, DynSubscriber, PubSubChannel, Publisher, Subscriber, WaitResult},
};

use super::{Receiver, Sender};

#[cfg(feature = "embedded-svc")]
pub use embedded_svc_impl::*;

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

#[cfg(feature = "embedded-svc")]
pub mod embedded_svc_impl {
    pub struct SvcSender<S>(S);

    impl<S> SvcSender<S>
    where
        S: embedded_svc::event_bus::asynch::Sender,
    {
        pub const fn new(sender: S) -> Self {
            Self(sender)
        }

        pub async fn send(&mut self, data: S::Data) -> Result<(), S::Error> {
            self.0.send(data).await
        }
    }

    impl<S> crate::asynch::Sender for SvcSender<S>
    where
        S: embedded_svc::event_bus::asynch::Sender,
    {
        type Error = S::Error;

        type Data = S::Data;

        async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
            SvcSender::send(self, data).await
        }
    }

    pub struct SvcReceiver<R>(R);

    impl<R> SvcReceiver<R>
    where
        R: embedded_svc::event_bus::asynch::Receiver,
    {
        pub const fn new(ws_receiver: R) -> Self {
            Self(ws_receiver)
        }

        pub async fn recv(&mut self) -> Result<R::Data, R::Error> {
            self.0.recv().await
        }
    }

    impl<R> crate::asynch::Receiver for SvcReceiver<R>
    where
        R: embedded_svc::event_bus::asynch::Receiver,
    {
        type Error = R::Error;

        type Data = R::Data;

        async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
            SvcReceiver::recv(self).await
        }
    }
}
