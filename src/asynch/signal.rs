use core::{convert::Infallible, future::Future};

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};

use super::{Receiver, Sender};

impl<'t, M, T> Sender for &'t Signal<M, T>
where
    M: RawMutex + 't,
    T: Send + 't,
{
    type Error = Infallible;

    type Data = T;

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where Self: 'a;

    fn send(&mut self, data: Self::Data) -> Self::SendFuture<'_> {
        async move {
            Signal::signal(self, data);

            Ok(())
        }
    }
}

impl<'t, M, T> Receiver for &'t Signal<M, T>
where
    M: RawMutex + 't,
    T: Send + 't,
{
    type Error = Infallible;

    type Data = T;

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> + 'a
    where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move { Ok(Signal::wait(self).await) }
    }
}
