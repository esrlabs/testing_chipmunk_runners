use crate::ByteSource;
use crate::{Error as SourceError, ReloadInfo, SourceFilter};
use async_trait::async_trait;
use buf_redux::Buffer;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

const TIMEOUT: u64 = 100;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerialError {
    #[error("Serial setup problem: {0}")]
    Setup(String),
    #[error("Unrecoverable serial error: {0}")]
    Unrecoverable(String),
}

pub struct SerialSource {
    port: SerialStream,
    buffer: Buffer,
    read: [u8; 1],
    amount: usize,
}

impl SerialSource {
    pub fn new(path: &str, baud_rate: u32) -> Result<Self, SerialError> {
        Ok(Self {
            buffer: Buffer::new(),
            read: [0; 1],
            amount: 0,
            port: tokio_serial::new(path, baud_rate)
                .open_native_async()
                .map_err(|e| SerialError::Unrecoverable(format!("Could not open port: {}", e)))?,
        })
    }
}

#[async_trait]
impl ByteSource for SerialSource {
    async fn reload(
        &mut self,
        _filter: Option<&SourceFilter>,
    ) -> Result<Option<ReloadInfo>, SourceError> {
        let mut copied: usize = 0;
        let mut buffer_error: String = "".to_string();
        let timeout = tokio::time::timeout(Duration::from_millis(TIMEOUT), async {
            loop {
                if let Err(err) = self.port.read_exact(&mut self.read).await {
                    buffer_error = format!("read from serial stream failed: {}", err);
                };
                copied += self.buffer.copy_from_slice(&self.read);
            }
        });
        match timeout.await {
            Ok(_) => {
                if buffer_error.is_empty() {
                    self.amount = copied;
                    Ok(Some(ReloadInfo::new(self.amount, self.amount, 0, None)))
                } else {
                    Err(SourceError::Unrecoverable(buffer_error))
                }
            }
            Err(_) => Ok(Some(ReloadInfo::new(0, 0, 0, None))),
        }
    }

    fn current_slice(&self) -> &[u8] {
        self.buffer.buf()
    }

    fn consume(&mut self, offset: usize) {
        self.buffer.consume(offset);
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/*
#[tokio::test]
async fn test_serial() {
    // Skipped for now due to permission issue, will be fixed later on
    let mut sender = "";
    let mut receiver = "";
    let baud_rate = 9600;
    if cfg!(windows) {
        sender = "COM10";
        receiver = "COM11";
    } else if cfg!(unix) {
        sender = "/dev/ttyS11";
        receiver = "/dev/ttyS12";
    }
    let text_parser = parsers::text::StringTokenizer;
    let serial_source = SerialSource::new(receiver, baud_rate).expect("create SerialSource failed");
    let mut text_msg_producer = crate::producer::MessageProducer::new(text_parser, serial_source);
    let msg_stream = text_msg_producer.as_stream();
    futures::pin_mut!(msg_stream);
    let messages = [
        "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
    ];
    let mut sender = tokio_serial::new(sender, baud_rate)
        .open_native_async()
        .expect("open port failed");
    tokio::select! {
        _ = async {
            for message in messages {
                sender.writable().await.expect("send message not possible");
                sender.try_write(format!("{}\n", &message).as_bytes()).expect("send message failed");
                tokio::time::sleep(Duration::from_millis(TIMEOUT)).await;
            }
        } => (),
        _ = async {
            let mut index = 0;
            while let Some(item) = tokio_stream::StreamExt::next(&mut msg_stream).await {
                if let (_, parsers::MessageStreamItem::Item(v)) = item {
                    assert_eq!(messages[index], v.to_string());
                    index += 1;
                }
            }
        } => (),
    }
}
*/