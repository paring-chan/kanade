use std::{sync::Arc, time::Duration};

use api_types::AgentLogMessage;
use futures_util::SinkExt;
use tokio::{net::TcpStream, sync::mpsc, time::interval};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::config::AgentConfig;

pub struct LogSender {
    pub sender: mpsc::Sender<AgentLogMessage>,
    _task: tokio::task::JoinHandle<()>,
}

impl LogSender {
    pub fn new(config: Arc<AgentConfig>) -> Self {
        let (tx, mut rx) = mpsc::channel(128);

        let uri = config
            .api_uri
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1);
        let ws_url = format!("{}/_/ws/agent", uri);

        Self {
            sender: tx,
            _task: tokio::spawn(async move {
                let mut buffer = Vec::<AgentLogMessage>::with_capacity(50);
                let mut ticker = interval(Duration::from_millis(100));

                let mut ws_stream: Option<WebSocketStream<_>> = None;

                loop {
                    tokio::select! {
                        maybe_msg = rx.recv() => {
                            match maybe_msg {
                                Some(msg) => {
                                    buffer.push(msg);
                                    if buffer.len() >= 50 {
                                        Self::flush_buffer(&ws_url, &mut ws_stream, &mut buffer).await;
                                    }
                                }
                                None => {
                                    break
                                },
                            }
                        }
                        _ = ticker.tick(), if !buffer.is_empty() => {
                            Self::flush_buffer(&ws_url, &mut ws_stream, &mut buffer).await;
                        }
                    }
                }

                Self::flush_buffer(&ws_url, &mut ws_stream, &mut buffer).await;

                if let Some(mut stream) = ws_stream {
                    _ = stream
                        .close(None)
                        .await
                        .inspect_err(|e| error!("ws close err: {e:?}"));
                }
            }),
        }
    }

    async fn flush_buffer(
        ws_url: &str,
        stream: &mut Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        buffer: &mut Vec<AgentLogMessage>,
    ) {
        if stream.is_none() {
            match connect_async(ws_url).await {
                Ok(ws) => {
                    *stream = Some(ws.0);
                }
                Err(e) => {
                    error!("failed to open new stream: {e:?}");
                    return;
                }
            }
        };

        let conn = stream.as_mut().unwrap();

        let data = match serde_json::to_string(buffer) {
            Ok(data) => data,
            Err(err) => {
                error!("failed to serialize ws json: {err:?}");
                return;
            }
        };

        match conn.send(Message::Text(data.into())).await {
            Ok(_) => {}
            Err(e) => {
                error!("failed to send ws message: {e:?}");

                *stream = None;

                // block overflow
                buffer.truncate(300);

                return;
            }
        };

        buffer.clear();
    }

    // pub async fn close(self) -> anyhow::Result<()> {
    //     drop(self.sender);
    //     self.task.await?;
    //     Ok(())
    // }
}
