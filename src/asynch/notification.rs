use core::{convert::Infallible, future::Future};

use crate::notification::Notification;

use super::{Receiver, Sender};

impl<'t> Sender for &'t Notification {
    type Error = Infallible;

    type Data = ();

    type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where Self: 'a;

    fn send(&mut self, _data: Self::Data) -> Self::SendFuture<'_> {
        async move {
            Notification::notify(self);

            Ok(())
        }
    }
}

impl<'t> Receiver for &'t Notification {
    type Error = Infallible;

    type Data = ();

    type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> + 'a
    where Self: 'a;

    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async move { Ok(Notification::wait(self).await) }
    }
}
