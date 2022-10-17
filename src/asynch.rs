use core::fmt::Debug;
use core::future::Future;
use core::marker::PhantomData;

pub mod mpmc;
#[cfg(feature = "notification")]
pub mod notification;
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

pub struct Mapper<C, F, Q>(C, F, PhantomData<fn() -> Q>);

impl<C, F, Q> Mapper<C, F, Q> {
    pub const fn new(channel: C, mapper: F) -> Self {
        Self(channel, mapper, PhantomData)
    }
}

impl<C, F, Q> Sender for Mapper<C, F, Q>
where
    C: Sender,
    F: Fn(Q) -> Option<C::Data>,
{
    type Error = C::Error;

    type Data = Q;

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> where Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
        async move {
            if let Some(data) = (self.1)(data) {
                self.0.send(data).await
            } else {
                Ok(())
            }
        }
    }
}

impl<C, F, Q> Receiver for Mapper<C, F, Q>
where
    C: Receiver,
    F: Fn(C::Data) -> Option<Q>,
{
    type Error = C::Error;

    type Data = Q;

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move {
            loop {
                if let Some(data) = (self.1)(self.0.recv().await?) {
                    return Ok(data);
                }
            }
        }
    }
}
