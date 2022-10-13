#[cfg(any(feature = "edge-net", feature = "embedded-svc", feature = "embedded-svc-prost"))]
pub use error::*;

#[cfg(feature = "edge-net")]
pub use edge_net_impl::*;

#[cfg(any(feature = "embedded-svc", feature = "embedded-svc-prost"))]
pub use embedded_svc_impl::*;

#[cfg(any(feature = "edge-net", feature = "embedded-svc", feature = "embedded-svc-prost"))]
mod error {
    use core::fmt::{self, Debug, Display};

    #[cfg(feature = "embedded-svc-prost")]
    #[derive(Debug)]
    pub enum ProstError {
        Encode(prost::EncodeError),
        Decode(prost::DecodeError),
    }

    #[cfg(feature = "embedded-svc-prost")]
    impl Display for ProstError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProstError::Encode(e) => write!(f, "[Encode]: {}", e),
            ProstError::Decode(e) => write!(f, "[Decode]: {}", e),
        }
    }
    }

    #[derive(Debug)]
    pub enum WsError<E> {
        IoError(E),
        UnknownFrameError,
        #[cfg(any(feature = "edge-net", feature = "embedded-svc"))]
        PostcardError(postcard::Error),
        #[cfg(feature = "embedded-svc-prost")]
        ProstError(ProstError),
    }

    impl<E> Display for WsError<E>
    where
        E: Display,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::IoError(e) => write!(f, "IO Error: {}", e),
                Self::UnknownFrameError => write!(f, "Unknown Frame Error"),
                #[cfg(any(feature = "edge-net", feature = "embedded-svc"))]
                Self::PostcardError(e) => write!(f, "Postcard Error: {}", e),
                #[cfg(feature = "embedded-svc-prost")]
                Self::ProstError(e) => write!(f, "Prost Error {}", e)
            }
        }
    }

    #[cfg(feature = "std")]
    impl<E> std::error::Error for WsError<E> where E: Display + Debug {}

    #[cfg(any(feature = "edge-net", feature = "embedded-svc"))]
    impl<E> From<postcard::Error> for WsError<E> {
        fn from(e: postcard::Error) -> Self {
            WsError::PostcardError(e)
        }
    }

    #[cfg(feature = "embedded-svc-prost")]
    impl<E> From<prost::EncodeError> for WsError<E> {
        fn from(e: prost::EncodeError) -> Self {
            WsError::ProstError(ProstError::Encode(e))
        }
    }

    #[cfg(feature = "embedded-svc-prost")]
    impl<E> From<prost::DecodeError> for WsError<E> {
        fn from(e: prost::DecodeError) -> Self {
            WsError::ProstError(ProstError::Decode(e))
        }
    }
}

#[cfg(feature = "edge-net")]
mod edge_net_impl {
    use core::future::Future;
    use core::marker::PhantomData;

    use embedded_io::asynch::{Read, Write};

    use serde::{de::DeserializeOwned, Serialize};

    use edge_net::asynch::ws::{self, FrameType};

    use super::*;

    pub struct WsSender<const N: usize, W, D>(W, Option<u32>, PhantomData<fn() -> D>);

    impl<const N: usize, W, D> WsSender<N, W, D> {
        pub const fn new(write: W, mask: Option<u32>) -> Self {
            Self(write, mask, PhantomData)
        }

        pub async fn send<'a>(&'a mut self, data: &'a D) -> Result<(), WsError<ws::Error<W::Error>>>
        where
            W: Write,
            D: Serialize,
        {
            let mut frame_buf = [0_u8; N];

            let frame_data = postcard::to_slice(data, &mut frame_buf)?;

            ws::send(&mut self.0, FrameType::Binary(false), self.1, frame_data)
                .await
                .map_err(WsError::IoError)?;

            Ok(())
        }
    }

    impl<const N: usize, W, D> crate::asynch::Sender for WsSender<N, W, D>
    where
        W: Write,
        D: Serialize,
    {
        type Error = WsError<ws::Error<W::Error>>;

        type Data = D;

        type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> where Self: 'a;

        fn send<'a>(&'a mut self, data: &'a Self::Data) -> Self::SendFuture<'a> {
            async move { WsSender::send(self, data).await }
        }
    }

    pub struct WsReceiver<const N: usize, R, D>(R, PhantomData<fn() -> D>);

    impl<const N: usize, R, D> WsReceiver<N, R, D> {
        pub const fn new(read: R) -> Self {
            Self(read, PhantomData)
        }

        pub async fn recv(&mut self) -> Result<Option<D>, WsError<ws::Error<R::Error>>>
        where
            R: Read,
            D: DeserializeOwned,
        {
            let mut frame_buf = [0_u8; N];

            let (frame_type, frame_buf) = loop {
                let (frame_type, size) = ws::recv(&mut self.0, &mut frame_buf)
                    .await
                    .map_err(WsError::IoError)?;

                if frame_type != FrameType::Ping && frame_type != FrameType::Pong {
                    break (frame_type, &frame_buf[..size]);
                }
            };

            match frame_type {
                FrameType::Text(_) | FrameType::Continue(_) => Err(WsError::UnknownFrameError),
                FrameType::Binary(_) => Ok(Some(
                    postcard::from_bytes(frame_buf).map_err(WsError::PostcardError)?,
                )),
                FrameType::Close => Ok(None),
                _ => unreachable!(),
            }
        }
    }

    impl<const N: usize, R, D> crate::asynch::Receiver for WsReceiver<N, R, D>
    where
        R: Read,
        D: DeserializeOwned,
    {
        type Error = WsError<ws::Error<R::Error>>;

        type Data = Option<D>;

        type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            async move { WsReceiver::recv(self).await }
        }
    }
}

#[cfg(any(feature = "embedded-svc", feature = "embedded-svc-prost"))]
pub mod embedded_svc_impl {
    use core::fmt::Debug;
    use core::future::Future;
    use core::marker::PhantomData;

    use log::{info, warn};

    use embassy_sync::blocking_mutex::raw::NoopRawMutex;

    use embedded_svc::ws::asynch::server::Acceptor;
    use embedded_svc::ws::{self, FrameType};

    use super::*;

    #[cfg(feature = "embedded-svc")]
    use serde::Serialize as SendData;
    #[cfg(feature = "embedded-svc")]
    use serde::de::DeserializeOwned as ReceiveData;

    #[cfg(feature = "embedded-svc-prost")]
    use prost::Message as SendData;
    #[cfg(feature = "embedded-svc-prost")]
    pub trait ReceiveData: prost::Message + Default {}
    #[cfg(feature = "embedded-svc-prost")]
    impl<T: prost::Message + Default> ReceiveData for T {}

    pub struct WsSvcSender<const N: usize, S, D>(S, PhantomData<fn() -> D>);

    impl<const N: usize, S, D> WsSvcSender<N, S, D> {
        pub const fn new(ws_sender: S) -> Self {
            Self(ws_sender, PhantomData)
        }

        pub async fn send<'a>(&'a mut self, data: &'a D) -> Result<(), WsError<S::Error>>
        where
            S: embedded_svc::ws::asynch::Sender,
            D: SendData,
        {
            let mut frame_buf = [0_u8; N];

            #[cfg(feature = "embedded-svc")]
            let frame_data = postcard::to_slice(data, &mut frame_buf)?;
            #[cfg(feature = "embedded-svc-prost")]
            let frame_data = {
                data.encode(&mut frame_buf.as_mut()).map_err(|e| WsError::from(e))?;
                &mut frame_buf[..data.encoded_len()]
            };

            self.0
                .send(FrameType::Binary(false), frame_data)
                .await
                .map_err(WsError::IoError)?;

            Ok(())
        }
    }

    impl<const N: usize, S, D> crate::asynch::Sender for WsSvcSender<N, S, D>
    where
        S: ws::asynch::Sender,
        D: SendData,
    {
        type Error = WsError<S::Error>;

        type Data = D;

        type SendFuture<'a> = impl Future<Output = Result<(), Self::Error>> where Self: 'a;

        fn send<'a>(&'a mut self, data: &'a Self::Data) -> Self::SendFuture<'a> {
            async move { WsSvcSender::send(self, data).await }
        }
    }

    pub struct WsSvcReceiver<const N: usize, R, D>(R, PhantomData<fn() -> D>);

    impl<const N: usize, R, D> WsSvcReceiver<N, R, D> {
        pub const fn new(ws_receiver: R) -> Self {
            Self(ws_receiver, PhantomData)
        }

        pub async fn recv(&mut self) -> Result<Option<D>, WsError<R::Error>>
        where
            R: embedded_svc::ws::asynch::Receiver,
            D: ReceiveData,
        {
            let mut frame_buf = [0_u8; N];

            let (frame_type, frame_buf) = loop {
                let (frame_type, size) = self
                    .0
                    .recv(&mut frame_buf)
                    .await
                    .map_err(WsError::IoError)?;

                if frame_type != FrameType::Ping && frame_type != FrameType::Pong {
                    break (frame_type, &frame_buf[..size]);
                }
            };

            match frame_type {
                FrameType::Text(_) | FrameType::Continue(_) => Err(WsError::UnknownFrameError),
                FrameType::Binary(_) => Ok(Some(
                    #[cfg(feature = "embedded-svc")]
                    postcard::from_bytes(frame_buf).map_err(WsError::PostcardError)?,
                    
                    #[cfg(feature = "embedded-svc-prost")]
                    prost::Message::decode(frame_buf).map_err(|e| WsError::from(e))?
                )),
                FrameType::Close | FrameType::SocketClose => Ok(None),
                _ => unreachable!(),
            }
        }
    }

    impl<const N: usize, R, D> crate::asynch::Receiver for WsSvcReceiver<N, R, D>
    where
        R: ws::asynch::Receiver,
        D: ReceiveData,
    {
        type Error = WsError<R::Error>;

        type Data = Option<D>;

        type RecvFuture<'a> = impl Future<Output = Result<Self::Data, Self::Error>> where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            async move { WsSvcReceiver::recv(self).await }
        }
    }

    pub trait AcceptorHandler {
        type SendData;
        type ReceiveData;

        type HandleFuture<'a, S, R>: Future<Output = Result<(), S::Error>>
        where
            Self: 'a,
            S: crate::asynch::Sender<Data = Self::SendData> + 'a,
            R: crate::asynch::Receiver<Error = S::Error, Data = Option<Self::ReceiveData>> + 'a,
            S::Error: Debug + 'a;

        fn handle<'a, S, R>(
            &'a self,
            sender: S,
            receiver: R,
            index: usize,
        ) -> Self::HandleFuture<'a, S, R>
        where
            S: crate::asynch::Sender<Data = Self::SendData> + 'a,
            R: crate::asynch::Receiver<Error = S::Error, Data = Option<Self::ReceiveData>> + 'a,
            S::Error: Debug + 'a;
    }

    pub async fn accept<const N: usize, const W: usize, const F: usize, A, H>(
        acceptor: A,
        handler: H,
    ) where
        A: Acceptor,
        H: AcceptorHandler,
        H::SendData: SendData,
        H::ReceiveData: ReceiveData,
    {
        info!("Creating queue for {} tasks and {} workers", W, N);
        let channel = embassy_sync::channel::Channel::<NoopRawMutex, _, W>::new();

        let mut workers = heapless::Vec::<_, N>::new();

        for index in 0..N {
            let channel = &channel;

            workers
                .push({
                    let handler = &handler;

                    async move {
                        loop {
                            let (sender, receiver) = channel.recv().await;

                            info!("Handler {}: Got new connection", index);

                            let res = handler
                                .handle(
                                    WsSvcSender::<F, _, _>::new(sender),
                                    WsSvcReceiver::<F, _, _>::new(receiver),
                                    index,
                                )
                                .await;

                            match res {
                                Ok(()) => {
                                    info!("Handler {}: connection closed", index);
                                }
                                Err(e) => {
                                    warn!(
                                        "Handler {}: connection closed with error {:?}",
                                        index, e
                                    );
                                }
                            }
                        }
                    }
                })
                .unwrap_or_else(|_| unreachable!());
        }

        let workers = workers.into_array::<N>().unwrap_or_else(|_| unreachable!());

        embassy_futures::select::select(
            async {
                loop {
                    info!("Acceptor: waiting for new connection");

                    match acceptor.accept().await {
                        Ok((sender, receiver)) => {
                            info!("Acceptor: got new connection");
                            channel.send((sender, receiver)).await;
                            info!("Acceptor: connection sent");
                        }
                        Err(e) => {
                            warn!("Got error when accepting a new connection: {:?}", e);
                        }
                    }
                }
            },
            embassy_futures::select::select_array(workers),
        )
        .await;

        info!("Server processing loop quit");
    }
}
