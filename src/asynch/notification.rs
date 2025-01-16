use core::convert::Infallible;

use crate::notification::Notification;

use super::{Receiver, Sender};

impl Sender for &Notification {
    type Error = Infallible;

    type Data = ();

    async fn send(&mut self, _data: Self::Data) -> Result<Self::Data, Self::Error> {
        Notification::notify(self);

        Ok(())
    }
}

impl Receiver for &Notification {
    type Error = Infallible;

    type Data = ();

    async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        Notification::wait(self).await;
        Ok(())
    }
}
