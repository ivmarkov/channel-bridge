use core::fmt::{self, Debug, Display};

#[cfg(feature = "edge-ws")]
pub use edge_ws_impl::*;

#[cfg(feature = "embedded-svc")]
pub use embedded_svc_impl::*;

#[cfg(feature = "wasm")]
pub use wasm_impl::*;

#[cfg(not(feature = "prost"))]
use serde::de::DeserializeOwned as ReceiveData;
#[cfg(not(feature = "prost"))]
use serde::Serialize as SendData;

#[cfg(feature = "prost")]
use prost::Message as SendData;
#[cfg(feature = "prost")]
pub trait ReceiveData: prost::Message + Default {}
#[cfg(feature = "prost")]
impl<T: prost::Message + Default> ReceiveData for T {}

pub const DEFAULT_HANDLER_TASKS_COUNT: usize = 4;
pub const DEFAULT_BUF_SIZE: usize = 4096;

#[cfg(feature = "prost")]
#[derive(Debug)]
pub enum ProstError {
    Encode(prost::EncodeError),
    Decode(prost::DecodeError),
}

#[cfg(feature = "prost")]
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
    #[cfg(not(feature = "prost"))]
    PostcardError(postcard::Error),
    #[cfg(feature = "prost")]
    ProstError(ProstError),
    OversizedFrame(usize),
}

impl<E> Display for WsError<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO Error: {e}"),
            Self::UnknownFrameError => write!(f, "Unknown Frame Error"),
            #[cfg(not(feature = "prost"))]
            Self::PostcardError(e) => write!(f, "Postcard Error: {e}"),
            #[cfg(feature = "prost")]
            Self::ProstError(e) => write!(f, "Prost Error {e}"),
            Self::OversizedFrame(delta) => write!(
                f,
                "Oversized Frame Error: Frame exceeds max size by {delta}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl<E> std::error::Error for WsError<E> where E: Display + Debug {}

#[cfg(not(feature = "prost"))]
impl<E> From<postcard::Error> for WsError<E> {
    fn from(e: postcard::Error) -> Self {
        WsError::PostcardError(e)
    }
}

#[cfg(feature = "prost")]
impl<E> From<prost::EncodeError> for WsError<E> {
    fn from(e: prost::EncodeError) -> Self {
        WsError::ProstError(ProstError::Encode(e))
    }
}

#[cfg(feature = "prost")]
impl<E> From<prost::DecodeError> for WsError<E> {
    fn from(e: prost::DecodeError) -> Self {
        WsError::ProstError(ProstError::Decode(e))
    }
}

#[cfg(feature = "edge-ws")]
mod edge_ws_impl {
    use core::marker::PhantomData;

    use embedded_io_async::{Read, Write};

    use edge_ws::{io, FrameType};

    use super::*;

    pub struct WsSender<'a, W, D> {
        write: W,
        buf: &'a mut [u8],
        mask: Option<u32>,
        _type: PhantomData<fn() -> D>,
    }

    impl<'a, W, D> WsSender<'a, W, D>
    where
        W: Write,
        D: SendData,
    {
        pub fn new(write: W, buf: &'a mut [u8], mask: Option<u32>) -> Self {
            Self {
                write,
                buf,
                mask,
                _type: PhantomData,
            }
        }

        pub async fn send(&mut self, data: D) -> Result<(), WsError<io::Error<W::Error>>> {
            #[cfg(not(feature = "prost"))]
            let frame_data = postcard::to_slice(&data, self.buf)?;
            #[cfg(feature = "prost")]
            let frame_data = {
                data.encode(self.buf).map_err(WsError::from)?;
                &self.buf[..data.encoded_len()]
            };

            io::send(
                &mut self.write,
                FrameType::Binary(false),
                self.mask,
                frame_data,
            )
            .await
            .map_err(WsError::IoError)?;

            Ok(())
        }
    }

    impl<'a, W, D> crate::asynch::Sender for WsSender<'a, W, D>
    where
        W: Write,
        D: SendData,
    {
        type Error = WsError<io::Error<W::Error>>;

        type Data = D;

        async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
            WsSender::send(self, data).await
        }
    }

    pub struct WsReceiver<'a, R, D> {
        read: R,
        buf: &'a mut [u8],
        _type: PhantomData<fn() -> D>,
    }

    impl<'a, R, D> WsReceiver<'a, R, D>
    where
        R: Read,
        D: ReceiveData,
    {
        pub fn new(read: R, buf: &'a mut [u8]) -> Self {
            Self {
                read,
                buf,
                _type: PhantomData,
            }
        }

        pub async fn recv(&mut self) -> Result<Option<D>, WsError<io::Error<R::Error>>> {
            let (frame_type, frame_buf) = loop {
                let (frame_type, size) = io::recv(&mut self.read, self.buf)
                    .await
                    .map_err(WsError::IoError)?;

                if frame_type != FrameType::Ping && frame_type != FrameType::Pong {
                    if size > self.buf.len() {
                        return Err(WsError::OversizedFrame(size - self.buf.len()));
                    }
                    break (frame_type, &self.buf[..size]);
                }
            };

            match frame_type {
                FrameType::Text(_) | FrameType::Continue(_) => Err(WsError::UnknownFrameError),
                FrameType::Binary(_) => Ok(Some(
                    #[cfg(not(feature = "prost"))]
                    postcard::from_bytes(frame_buf).map_err(WsError::PostcardError)?,
                    #[cfg(feature = "prost")]
                    prost::Message::decode(frame_buf).map_err(WsError::from)?,
                )),
                FrameType::Close => Ok(None),
                _ => unreachable!(),
            }
        }
    }

    impl<'a, R, D> crate::asynch::Receiver for WsReceiver<'a, R, D>
    where
        R: Read,
        D: ReceiveData,
    {
        type Error = WsError<io::Error<R::Error>>;

        type Data = Option<D>;

        async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
            WsReceiver::recv(self).await
        }
    }
}

#[cfg(feature = "embedded-svc")]
pub mod embedded_svc_impl {
    use core::marker::PhantomData;
    use core::mem::MaybeUninit;
    use core::pin::pin;

    use log::{info, warn};

    use embassy_sync::blocking_mutex::raw::NoopRawMutex;

    use embedded_svc::ws::asynch::server;
    use embedded_svc::ws::{self, FrameType};

    use super::*;

    pub struct WsSvcSender<'a, S, D> {
        ws_sender: S,
        buf: &'a mut [u8],
        _type: PhantomData<fn() -> D>,
    }

    impl<'a, S, D> WsSvcSender<'a, S, D>
    where
        S: embedded_svc::ws::asynch::Sender,
        D: SendData,
    {
        pub fn new(ws_sender: S, buf: &'a mut [u8]) -> Self {
            Self {
                ws_sender,
                buf,
                _type: PhantomData,
            }
        }

        pub async fn send(&mut self, data: &D) -> Result<(), WsError<S::Error>> {
            #[cfg(not(feature = "prost"))]
            let frame_data = postcard::to_slice(data, self.buf)?;
            #[cfg(feature = "prost")]
            let frame_data = {
                data.encode(self.buf).map_err(WsError::from)?;
                &self.buf[..data.encoded_len()]
            };

            self.ws_sender
                .send(FrameType::Binary(false), frame_data)
                .await
                .map_err(WsError::IoError)?;

            Ok(())
        }
    }

    impl<'a, S, D> crate::asynch::Sender for WsSvcSender<'a, S, D>
    where
        S: ws::asynch::Sender,
        D: SendData,
    {
        type Error = WsError<S::Error>;

        type Data = D;

        async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
            WsSvcSender::send(self, &data).await
        }
    }

    pub struct WsSvcReceiver<'a, R, D> {
        ws_receiver: R,
        buf: &'a mut [u8],
        _type: PhantomData<fn() -> D>,
    }

    impl<'a, R, D> WsSvcReceiver<'a, R, D>
    where
        R: embedded_svc::ws::asynch::Receiver,
        D: ReceiveData,
    {
        pub fn new(ws_receiver: R, buf: &'a mut [u8]) -> Self {
            Self {
                ws_receiver,
                buf,
                _type: PhantomData,
            }
        }

        pub async fn recv(&mut self) -> Result<Option<D>, WsError<R::Error>> {
            let (frame_type, frame_buf) = loop {
                let (frame_type, size) = self
                    .ws_receiver
                    .recv(self.buf)
                    .await
                    .map_err(WsError::IoError)?;

                if frame_type != FrameType::Ping && frame_type != FrameType::Pong {
                    if size > self.buf.len() {
                        return Err(WsError::OversizedFrame(size - self.buf.len()));
                    }
                    break (frame_type, &self.buf[..size]);
                }
            };

            match frame_type {
                FrameType::Text(_) | FrameType::Continue(_) => Err(WsError::UnknownFrameError),
                FrameType::Binary(_) => Ok(Some(
                    #[cfg(not(feature = "prost"))]
                    postcard::from_bytes(frame_buf).map_err(WsError::PostcardError)?,
                    #[cfg(feature = "prost")]
                    prost::Message::decode(frame_buf).map_err(WsError::from)?,
                )),
                FrameType::Close | FrameType::SocketClose => Ok(None),
                _ => unreachable!(),
            }
        }
    }

    impl<'a, R, D> crate::asynch::Receiver for WsSvcReceiver<'a, R, D>
    where
        R: ws::asynch::Receiver,
        D: ReceiveData,
    {
        type Error = WsError<R::Error>;

        type Data = Option<D>;

        async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
            WsSvcReceiver::recv(self).await
        }
    }

    pub trait AcceptorHandler {
        type SendData;
        type ReceiveData;

        async fn handle<S, R>(
            &self,
            sender: S,
            receiver: R,
            task_id: usize,
        ) -> Result<(), S::Error>
        where
            S: crate::asynch::Sender<Data = Self::SendData>,
            R: crate::asynch::Receiver<Error = S::Error, Data = Option<Self::ReceiveData>>;
    }

    pub type DefaultAcceptor = Acceptor<{ DEFAULT_HANDLER_TASKS_COUNT }, { DEFAULT_BUF_SIZE }, 2>;

    pub struct Acceptor<
        const P: usize = DEFAULT_HANDLER_TASKS_COUNT,
        const B: usize = DEFAULT_BUF_SIZE,
        const W: usize = 2,
    >([MaybeUninit<[u8; B]>; P], [MaybeUninit<[u8; B]>; P]);

    impl<const P: usize, const B: usize, const W: usize> Acceptor<P, B, W> {
        #[inline(always)]
        pub const fn new() -> Self {
            Self([MaybeUninit::uninit(); P], [MaybeUninit::uninit(); P])
        }

        #[inline(never)]
        #[cold]
        pub async fn run<A, H>(&mut self, acceptor: A, handler: H)
        where
            A: server::Acceptor,
            H: AcceptorHandler,
            H::SendData: SendData,
            H::ReceiveData: ReceiveData,
        {
            info!("Creating queue for {W} tasks");
            let channel = embassy_sync::channel::Channel::<NoopRawMutex, _, W>::new();

            let mut workers = heapless::Vec::<_, { P }>::new();

            for task_id in 0..P {
                let channel = &channel;

                workers
                    .push({
                        let handler = &handler;
                        let send_buf = self.0[task_id].as_mut_ptr();
                        let recv_buf = self.1[task_id].as_mut_ptr();

                        async move {
                            loop {
                                let (sender, receiver) = channel.receive().await;

                                info!("Handler task {}: Got new connection", task_id);

                                let res = handler
                                    .handle(
                                        WsSvcSender::new(
                                            sender,
                                            unsafe { send_buf.as_mut() }.unwrap(),
                                        ),
                                        WsSvcReceiver::new(
                                            receiver,
                                            unsafe { recv_buf.as_mut() }.unwrap(),
                                        ),
                                        task_id,
                                    )
                                    .await;

                                match res {
                                    Ok(()) => {
                                        info!("Handler task {}: connection closed", task_id);
                                    }
                                    Err(e) => {
                                        warn!(
                                            "Handler task {}: connection closed with error {:?}",
                                            task_id, e
                                        );
                                    }
                                }
                            }
                        }
                    })
                    .unwrap_or_else(|_| unreachable!());
            }

            let acceptor = pin!(async {
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
            });

            embassy_futures::select::select(
                acceptor,
                embassy_futures::select::select_slice(&mut workers),
            )
            .await;

            info!("Server processing loop quit");
        }
    }
}

#[cfg(feature = "wasm")]
mod wasm_impl {
    use core::marker::PhantomData;

    use futures::stream::{SplitSink, SplitStream};
    use futures::{SinkExt, StreamExt};

    use gloo_net::websocket::{futures::WebSocket, Message, WebSocketError};

    use super::*;

    pub struct WsWebSender<D>(SplitSink<WebSocket, Message>, PhantomData<fn() -> D>);

    impl<D> WsWebSender<D>
    where
        D: SendData,
    {
        pub const fn new(sender: SplitSink<WebSocket, Message>) -> Self {
            Self(sender, PhantomData)
        }

        pub async fn send(&mut self, data: D) -> Result<(), WsError<WebSocketError>> {
            #[cfg(not(feature = "prost"))]
            let message =
                Message::Bytes(postcard::to_allocvec(&data).map_err(WsError::PostcardError)?);

            #[cfg(feature = "prost")]
            let message = Message::Bytes(data.encode_to_vec());

            self.0.send(message).await.map_err(WsError::IoError)
        }
    }

    impl<D> crate::asynch::Sender for WsWebSender<D>
    where
        D: SendData,
    {
        type Error = WsError<WebSocketError>;

        type Data = D;

        async fn send(&mut self, data: Self::Data) -> Result<(), Self::Error> {
            WsWebSender::send(self, data).await
        }
    }

    pub struct WsWebReceiver<D>(SplitStream<WebSocket>, PhantomData<fn() -> D>);

    impl<D> WsWebReceiver<D>
    where
        D: ReceiveData,
    {
        pub const fn new(receiver: SplitStream<WebSocket>) -> Self {
            Self(receiver, PhantomData)
        }

        pub async fn recv(&mut self) -> Result<Option<D>, WsError<WebSocketError>> {
            if let Some(message) = self.0.next().await {
                let message = message.map_err(WsError::IoError)?;

                if let Message::Bytes(bytes) = message {
                    #[cfg(not(feature = "prost"))]
                    let payload = postcard::from_bytes(&bytes).map_err(WsError::PostcardError)?;

                    #[cfg(feature = "prost")]
                    let payload = prost::Message::decode(&*bytes).map_err(WsError::from)?;

                    Ok(Some(payload))
                } else {
                    Err(WsError::UnknownFrameError)
                }
            } else {
                Ok(None)
            }
        }
    }

    impl<D> crate::asynch::Receiver for WsWebReceiver<D>
    where
        D: ReceiveData,
    {
        type Error = WsError<WebSocketError>;

        type Data = Option<D>;

        async fn recv(&mut self) -> Result<Self::Data, Self::Error> {
            WsWebReceiver::recv(self).await
        }
    }
}
