use std::sync::Arc;

use fred::prelude::*;
use secrecy::ExposeSecret;

use crate::config::AppConfig;

pub struct Realtime {
    client: Client,
    subscriber_client: Client,
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

        client.init().await?;
        subscriber_client.init().await?;

        client.on_error(|(error, server)| async move {
            error!(server = ?server, "valkey error: {error}");
            Ok(())
        });

        subscriber_client.on_error(|(error, server)| async move {
            error!(server = ?server, "subscriber error: {error}");
            Ok(())
        });

        subscriber_client.subscribe("kanade:events").await?;
        let mut message_stream = subscriber_client.message_rx();

        tokio::spawn(async move {
            info!("realtime receiver started");

            while let Ok(message) = message_stream.recv().await {
                debug!("{message:?}");
            }

            info!("realtime receiver stopped")
        });

        Ok(Self {
            client,
            subscriber_client,
        })
    }
}
