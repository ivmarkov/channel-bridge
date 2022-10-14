use core::fmt::Debug;
use core::future::Future;

pub mod mpmc;
pub mod pubsub;
pub mod signal;
pub mod ws;

pub trait Sender {
    type Error: Debug;

    type Data;

    type SendFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_>;
}

impl<'t, T> Sender for &'t mut T
where
    T: Sender + 't,
{
    type Error = T::Error;

    type Data = T::Data;

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> where Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
        async move { (*self).send(data).await }
    }
}

pub trait Receiver {
    type Error: Debug;

    type Data;

    type RecvFuture<'a>: Future<Output = Result<Self::Data, Self::Error>>
    where
        Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

impl<'t, T> Receiver for &'t mut T
where
    T: Receiver + 't,
{
    type Error = T::Error;

    type Data = T::Data;

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move { (*self).recv().await }
    }
}
