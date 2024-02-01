use core::fmt::Debug;
use core::marker::PhantomData;

pub mod mpmc;
pub mod notification;
pub mod pubsub;
pub mod signal;
#[cfg(any(feature = "edge-ws", feature = "embedded-svc", feature = "wasm"))]
pub mod ws;

pub trait Sender {
    type Error: Debug;

    type Data;

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error>;
}

impl<'t, T> Sender for &'t mut T
where
    T: Sender + 't,
{
    type Error = T::Error;

    type Data = T::Data;

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        (*self).send(data).await
    }
}

impl<D, E: Debug> Sender for PhantomData<fn() -> (D, E)> {
    type Error = E;

    type Data = D;

    async fn send(&mut self, _data: Self::Data) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn nil_sender<D, E: Debug>() -> impl Sender<Error = E, Data = D> {
    PhantomData
}

pub trait Receiver {
    type Error: Debug;

    type Data;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error>;
}

impl<'t, T> Receiver for &'t mut T
where
    T: Receiver + 't,
{
    type Error = T::Error;

    type Data = T::Data;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        (*self).recv().await
    }
}

impl<D, E: Debug> Receiver for PhantomData<fn() -> (D, E)> {
    type Error = E;

    type Data = D;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        core::future::pending().await
    }
}

pub fn nil_receiver<D, E: Debug>() -> impl Receiver<Error = E, Data = D> {
    PhantomData
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

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        if let Some(data) = (self.1)(data) {
            self.0.send(data).await
        } else {
            Ok(())
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

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        loop {
            if let Some(data) = (self.1)(self.0.recv().await?) {
                return Ok(data);
            }
        }
    }
}
