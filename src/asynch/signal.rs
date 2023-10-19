use core::convert::Infallible;

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};

use super::{Receiver, Sender};

impl<'t, M, T> Sender for &'t Signal<M, T>
where
    M: RawMutex + 't,
    T: Send + 't,
{
    type Error = Infallible;

    type Data = T;

    async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        Signal::signal(self, data);

        Ok(())
    }
}

impl<'t, M, T> Receiver for &'t Signal<M, T>
where
    M: RawMutex + 't,
    T: Send + 't,
{
    type Error = Infallible;

    type Data = T;

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(Signal::wait(self).await)
    }
}
