use std::net::SocketAddr;

use bytes::{BufMut, Bytes, BytesMut};
use futures::sync::mpsc;
use thrift::{
    protocol::{
        TBinaryOutputProtocol, TCompactOutputProtocol, TMessageIdentifier, TMessageType,
        TOutputProtocol,
    },
    transport::{ReadHalf, TBufferChannel, TIoChannel},
};
use tokio::net::UdpSocket;
use tokio::prelude::*;

use crate::{thrift_gen::agent, Process, Span};
use opentracing_rs_core::Tag;

pub trait Transport {
    fn append(&mut self, span: Span);
    fn flush(&mut self);
}

pub struct SpanBatch {
    pub(crate) process: Process,
    pub(crate) spans: Vec<Span>,
}

pub enum TransportProtocol {
    ThriftBinary,
    ThriftCompact,
}

pub struct ThriftEncoder {
    protocol: Box<TOutputProtocol + Send + Sync>,
    buffer: ReadHalf<TBufferChannel>,
    seq_number: i32,
}

impl ThriftEncoder {
    pub fn binary_protocol(buffer_size: usize) -> Self {
        let thrift_channel = TBufferChannel::with_capacity(0, buffer_size);
        let (read_buf, write_buf) = thrift_channel.split().unwrap();

        Self {
            protocol: Box::new(TBinaryOutputProtocol::new(write_buf, true)),
            buffer: read_buf,
            seq_number: 0,
        }
    }

    pub fn compact_protocol(buffer_size: usize) -> Self {
        let thrift_channel = TBufferChannel::with_capacity(0, buffer_size);
        let (read_buf, write_buf) = thrift_channel.split().unwrap();

        Self {
            protocol: Box::new(TCompactOutputProtocol::new(write_buf)),
            buffer: read_buf,
            seq_number: 0,
        }
    }

    // FIXME: convert to immutable function
    pub fn encode_span_batch(&mut self, batch: SpanBatch) -> Result<Bytes, ()> {
        let seq = {
            self.seq_number += 1;
            self.seq_number
        };

        {
            let message_ident = TMessageIdentifier::new("emitBatch", TMessageType::OneWay, seq);
            let call_args = agent::EmitBatchArgs {
                batch: From::from(batch),
            };
            self.protocol.write_message_begin(&message_ident).unwrap();
            call_args.write_to_out_protocol(&mut self.protocol).unwrap();
            self.protocol.write_message_end().unwrap();
        }

        let buf = self.buffer.write_bytes();
        self.buffer.empty_write_buffer();

        Ok(Bytes::from(buf))
    }
}

use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct UdpTransport {
    process: Arc<Process>,
    to_send: mpsc::UnboundedSender<Bytes>,
    span_buffer: Arc<RwLock<Vec<Span>>>,
    encoder: Arc<RwLock<ThriftEncoder>>,
}

impl UdpTransport {
    pub fn new(
        process: Process,
        to_send: mpsc::UnboundedSender<Bytes>,
        encoder: ThriftEncoder,
        buffer_size: usize,
    ) -> Self {
        let transport = Self {
            process: Arc::new(process),
            encoder: Arc::new(RwLock::new(encoder)),
            to_send: to_send,
            span_buffer: Arc::new(RwLock::new(Vec::with_capacity(buffer_size))),
        };

        transport
    }

    pub fn builder() -> UdpTransportBuilder {
        UdpTransportBuilder::default()
    }

    pub fn send_bytes(&mut self, data: &[u8]) -> impl Future<Item = (), Error = ()> {
        self.to_send
            .clone()
            .send(Bytes::from(data))
            .map(|_| ())
            .map_err(|_| ())
    }
}

impl Transport for UdpTransport {
    fn append(&mut self, span: Span) {
        {
            let mut buf = self.span_buffer.write().unwrap();
            buf.push(span);
            if buf.len() < buf.capacity() - 1 {
                return;
            }
        }

        self.flush();
    }

    fn flush(&mut self) {
        {
            let buf = self.span_buffer.read().unwrap();
            if buf.is_empty() {
                return;
            }
        }

        let batch = {
            let mut buf = self.span_buffer.write().unwrap();

            if buf.is_empty() {
                return;
            }

            SpanBatch {
                process: self.process.as_ref().clone(),
                spans: buf.drain(..).collect(),
            }
        };

        let buf = self
            .encoder
            .write()
            .unwrap()
            .encode_span_batch(batch)
            .unwrap();

        tokio::spawn(self.send_bytes(&buf));
    }
}

pub struct UdpStream {
    socket: UdpSocket,
    agent_addr: SocketAddr,
    to_send: mpsc::UnboundedReceiver<Bytes>,
    buffer: BytesMut,
}

impl UdpStream {
    pub fn new(agent_addr: SocketAddr, to_send: mpsc::UnboundedReceiver<Bytes>) -> Self {
        let socket = {
            let local_addr: SocketAddr = if agent_addr.is_ipv4() {
                "0.0.0.0:0"
            } else {
                "[::]:0"
            }
            .parse()
            .unwrap();
            UdpSocket::bind(&local_addr).expect("failed to bind UDP addr")
        };

        Self {
            socket,
            agent_addr,
            to_send,
            buffer: BytesMut::new(),
        }
    }

    fn buffer(&mut self, buf: &[u8]) {
        self.buffer.reserve(buf.len());
        self.buffer.put(buf);
    }

    fn poll_flush(&mut self) -> Poll<(), tokio::io::Error> {
        while !self.buffer.is_empty() {
            let n = try_ready!(self.socket.poll_send_to(&self.buffer, &self.agent_addr));

            assert!(n > 0);

            self.buffer.split_to(n);
        }

        Ok(Async::Ready(()))
    }
}

impl Stream for UdpStream {
    type Item = ();
    type Error = tokio::io::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        const TICK: u8 = 10;

        for _ in 0..TICK {
            match self.to_send.poll().unwrap() {
                Async::Ready(Some(data)) => self.buffer(&data),
                _ => break,
            }
        }

        self.poll_flush()?;

        Ok(Async::NotReady)
    }
}

pub struct UdpTransportBuilder {
    transport_protocol: Option<TransportProtocol>,
    service_name: Option<String>,
    encoding_buffer_size: usize,
    span_buffer_size: usize,
    tags: Option<Vec<Tag>>,
}

impl Default for UdpTransportBuilder {
    fn default() -> Self {
        Self {
            transport_protocol: None,
            service_name: None,
            encoding_buffer_size: 4096,
            span_buffer_size: 1000,
            tags: None,
        }
    }
}

impl UdpTransportBuilder {
    pub fn transport_protocol(mut self, protocol: TransportProtocol) -> Self {
        self.transport_protocol = Some(protocol);
        self
    }

    pub fn process_service_name<N>(mut self, service_name: N) -> Self
    where
        N: Into<String>,
    {
        self.service_name = Some(service_name.into());
        self
    }

    pub fn process_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn encoding_buffer_size(mut self, buffer_size: usize) -> Self {
        self.encoding_buffer_size = buffer_size;
        self
    }

    pub fn span_buffer_size(mut self, buffer_size: usize) -> Self {
        assert!(buffer_size > 1);
        self.span_buffer_size = buffer_size;
        self
    }

    pub fn build_and_serve(
        self,
        agent_addr: SocketAddr,
    ) -> (UdpTransport, impl Future<Item = (), Error = ()>) {
        let process = {
            let process_service_name = self
                .service_name
                .unwrap_or("opentracing-rs_service".to_owned());

            let process_tags = {
                let mut tags = self.tags.unwrap_or_default();

                tags.push(Tag::new(
                    crate::tag::JAEGER_CLIENT_VERSION_TAG_KEY,
                    crate::tag::JAEGER_CLIENT_VERSION,
                ));
                tags
            };

            Process::with_tags(process_service_name, process_tags)
        };

        let encoder = {
            let buffer_size = self.encoding_buffer_size;
            let protocol = self
                .transport_protocol
                .unwrap_or(TransportProtocol::ThriftBinary);

            match protocol {
                TransportProtocol::ThriftBinary => ThriftEncoder::binary_protocol(buffer_size),
                TransportProtocol::ThriftCompact => ThriftEncoder::compact_protocol(buffer_size),
            }
        };

        let (to_send_sender, to_send_receiver) = mpsc::unbounded();

        let stream = UdpStream::new(agent_addr, to_send_receiver);

        let transport = UdpTransport::new(process, to_send_sender, encoder, self.span_buffer_size);

        (transport, stream.for_each(|_| Ok(())).map_err(|_| ()))
    }
}
