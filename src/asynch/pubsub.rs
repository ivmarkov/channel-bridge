use core::convert::Infallible;
use core::future::Future;

use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{DynPublisher, DynSubscriber, Publisher, Subscriber, WaitResult},
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

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
        async move {
            self.publish(data.clone()).await;

            Ok(())
        }
    }
}

impl<'t, T> Receiver for DynSubscriber<'t, T>
where
    T: Clone + 't,
{
    type Error = Infallible;

    type Data = T;

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> + 'a
    where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move {
            match self.next_message().await {
                WaitResult::Message(data) => Ok(data),
                _ => panic!(),
            }
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

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
        async move {
            self.publish(data.clone()).await;

            Ok(())
        }
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

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> + 'a
    where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move {
            match self.next_message().await {
                WaitResult::Message(data) => Ok(data),
                _ => panic!(),
            }
        }
    }
}

#[cfg(feature = "embedded-svc")]
pub mod embedded_svc_impl {
    use core::future::Future;
    use core::marker::PhantomData;

    use super::*;

    pub struct SvcSender<S, D>(S, PhantomData<fn() -> D>);

    impl<S, D> SvcSender<S, D> {
        pub const fn new(sender: S) -> Self {
            Self(sender, PhantomData)
        }

        pub async fn send(&mut self, data: D)
        where
            S: embedded_svc::event_bus::asynch::Sender<Data = D>,
        {
            self.0.send(data).await;
        }
    }

    impl<S, D> crate::asynch::Sender for SvcSender<S, D>
    where
        S: embedded_svc::event_bus::asynch::Sender<Data = D>,
    {
        type Error = Infallible;

        type Data = D;

        type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
            async move {
                SvcSender::send(self, data).await;

                Ok(())
            }
        }
    }

    pub struct SvcReceiver<R, D>(R, PhantomData<fn() -> D>);

    impl<R, D> SvcReceiver<R, D> {
        pub const fn new(ws_receiver: R) -> Self {
            Self(ws_receiver, PhantomData)
        }

        pub async fn recv(&mut self) -> D
        where
            R: embedded_svc::event_bus::asynch::Receiver<Result = D>,
        {
            self.0.recv().await
        }
    }

    impl<R, D> crate::asynch::Receiver for SvcReceiver<R, D>
    where
        R: embedded_svc::event_bus::asynch::Receiver<Result = D>,
    {
        type Error = Infallible;

        type Data = D;

        type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> + 'a where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            async move { Ok(SvcReceiver::recv(self).await) }
        }
    }
}
