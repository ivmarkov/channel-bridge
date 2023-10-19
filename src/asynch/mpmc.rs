use core::convert::Infallible;

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

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        DynamicSender::send(self, data).await;

        Ok(())
    }
}

impl<'t, T> Receiver for DynamicReceiver<'t, T>
where
    T: 't,
{
    type Error = Infallible;

    type Data = T;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(DynamicReceiver::receive(self).await)
    }
}

impl<'t, M, T, const N: usize> Sender for embassy_sync::channel::Sender<'t, M, T, N>
where
    M: RawMutex + 't,
    T: 't,
{
    type Error = Infallible;

    type Data = T;

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        embassy_sync::channel::Sender::send(self, data).await;

        Ok(())
    }
}

impl<'t, M, T, const N: usize> Receiver for embassy_sync::channel::Receiver<'t, M, T, N>
where
    M: RawMutex + 't,
    T: 't,
{
    type Error = Infallible;

    type Data = T;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(embassy_sync::channel::Receiver::receive(self).await)
    }
}
