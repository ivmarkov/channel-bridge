use core::convert::Infallible;
use core::future::Future;

use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    channel::{DynamicReceiver, DynamicSender},
};

use super::{Receiver, Sender};

impl<'t, T> Sender for DynamicSender<'t, T>
where
    T: 't,
{
    type Error = Infallible;

    type Data = T;

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
        async move {
            DynamicSender::send(self, data).await;

            Ok(())
        }
    }
}

impl<'t, T> Receiver for DynamicReceiver<'t, T>
where
    T: 't,
{
    type Error = Infallible;

    type Data = T;

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> + 'a
    where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move { Ok(DynamicReceiver::recv(self).await) }
    }
}

impl<'t, M, T, const N: usize> Sender for embassy_sync::channel::Sender<'t, M, T, N>
where
    M: RawMutex + 't,
    T: 't,
{
    type Error = Infallible;

    type Data = T;

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
        async move {
            embassy_sync::channel::Sender::send(self, data).await;

            Ok(())
        }
    }
}

impl<'t, M, T, const N: usize> Receiver for embassy_sync::channel::Receiver<'t, M, T, N>
where
    M: RawMutex + 't,
    T: 't,
{
    type Error = Infallible;

    type Data = T;

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> + 'a
    where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move { Ok(embassy_sync::channel::Receiver::recv(self).await) }
    }
}
