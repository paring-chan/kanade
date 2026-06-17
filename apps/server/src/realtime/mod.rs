use std::sync::Arc;

use fred::prelude::*;
use secrecy::ExposeSecret;
use tokio::sync::broadcast;
use tracing::Instrument;

use crate::{
    config::AppConfig,
    realtime::types::{EventMessage, LogMessage},
};

pub mod types;

pub struct Realtime {
    pub client: Client,
    _subscriber_client: Client,
    _log_client: Client,

    pub event_stream: broadcast::Sender<EventMessage>,
    pub log_stream: broadcast::Sender<LogMessage>,
}

impl Realtime {
    pub async fn new(config: Arc<AppConfig>) -> crate::Result<Self> {
        let config = Config::from_url(config.valkey.url.expose_secret())?;
        let client = Builder::from_config(config)
            .with_connection_config(|config| {
                config.connection_timeout = std::time::Duration::from_secs(10);
                config.tcp.nodelay = Some(true);
            })
            .build()?;

        let subscriber_client = client.clone_new();
        let log_client = client.clone_new();

        client.init().await?;
        subscriber_client.init().await?;
        log_client.init().await?;

        client.on_error(|(error, server)| async move {
            error!(server = ?server, "valkey error: {error}");
            Ok(())
        });

        subscriber_client.on_error(|(error, server)| async move {
            error!(server = ?server, "subscriber error: {error}");
            Ok(())
        });

        log_client.on_error(|(error, server)| async move {
            error!(server = ?server, "log subscriber error: {error}");
            Ok(())
        });

        subscriber_client.on_reconnect({
            let subscriber_client = subscriber_client.clone();
            move |_| {
                let c = subscriber_client.clone();
                async move {
                    c.subscribe("kanade:events").await?;
                    Ok(())
                }
            }
        });

        log_client.on_reconnect({
            let log_client = log_client.clone();
            move |_| {
                let c = log_client.clone();
                async move {
                    c.subscribe("kanade:logs").await?;
                    Ok(())
                }
            }
        });

        subscriber_client.subscribe("kanade:events").await?;
        log_client.subscribe("kanade:logs").await?;
        let mut message_stream = subscriber_client.message_rx();

        let (event_sender, _) = broadcast::channel(100);

        tokio::spawn({
            let sender = event_sender.clone();
            async move {
                info!("realtime receiver started");

                while let Ok(message) = message_stream.recv().await {
                    let fred::types::Value::String(data) = message.value else {
                        warn!("invalid data received: {:?}", message.value);
                        continue;
                    };
                    let message: EventMessage = match serde_json::from_str(&data) {
                        Ok(message) => message,
                        Err(err) => {
                            warn!("json parse failed: {:?}", err);
                            continue;
                        }
                    };
                    debug!("realtime message: {message:?}");

                    _ = sender.send(message).inspect_err(|e| {
                        warn!("event send failed: {:?}", e);
                    });
                }

                info!("realtime receiver stopped")
            }
            .instrument(info_span!("event-receiver"))
        });

        let mut log_stream = log_client.message_rx();

        let (log_sender, _) = broadcast::channel(100);

        tokio::spawn({
            let sender = log_sender.clone();
            async move {
                info!("realtime receiver started");

                while let Ok(message) = log_stream.recv().await {
                    let fred::types::Value::String(data) = message.value else {
                        warn!("invalid data received: {:?}", message.value);
                        continue;
                    };
                    let message: LogMessage = match serde_json::from_str(&data) {
                        Ok(message) => message,
                        Err(err) => {
                            warn!("json parse failed: {:?}", err);
                            continue;
                        }
                    };
                    debug!("realtime message: {message:?}");

                    _ = sender.send(message).inspect_err(|e| {
                        warn!("event send failed: {:?}", e);
                    });
                }

                info!("realtime receiver stopped")
            }
            .instrument(info_span!("event-receiver"))
        });

        Ok(Self {
            client,
            _subscriber_client: subscriber_client,
            _log_client: log_client,
            event_stream: event_sender,
            log_stream: log_sender,
        })
    }

    pub async fn publish(&self, payload: &EventMessage) -> crate::Result<()> {
        let _: () = self
            .client
            .publish("kanade:events", serde_json::to_string(&payload)?)
            .await?;

        Ok(())
    }

    pub async fn publish_log(&self, payload: &LogMessage) -> crate::Result<()> {
        let _: () = self
            .client
            .publish("kanade:logs", serde_json::to_string(&payload)?)
            .await?;

        let _: () = self
            .client
            .xadd(
                format!("kanade:logs:{}", payload.job_id),
                false,
                None,
                "*",
                ("data", serde_json::to_string(&payload.entry)?),
            )
            .await?;

        Ok(())
    }
}
