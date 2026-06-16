use std::sync::Arc;

use api_types::AgentLogMessage;
use futures_util::{SinkExt, StreamExt};
use poem::{
    EndpointExt, IntoResponse, Route,
    endpoint::BoxEndpoint,
    get, handler,
    web::{
        Data,
        websocket::{Message, WebSocket},
    },
};

use crate::realtime::Realtime;

pub fn routes() -> BoxEndpoint<'static> {
    Route::new()
        .at("/events", get(events_ws))
        .at("/agent", get(handle_agent))
        .boxed()
}

#[handler]
async fn events_ws(ws: WebSocket, Data(realtime): Data<&Arc<Realtime>>) -> impl IntoResponse {
    let realtime = realtime.clone();
    ws.on_upgrade(move |socket| async move {
        let (mut sink, _) = socket.split();
        let mut receiver = realtime.event_stream.subscribe();

        while let Ok(msg) = receiver.recv().await {
            if let Ok(data) = serde_json::to_string(&msg)
                && let Err(_) = sink.send(Message::Text(data)).await
            {
                break;
            }
        }
    })
}

#[handler]
async fn handle_agent(ws: WebSocket, Data(realtime): Data<&Arc<Realtime>>) -> impl IntoResponse {
    let realtime = realtime.clone();
    ws.on_upgrade(move |mut socket| async move {
        // let mut rt = realtime.event_stream.subscribe();

        while let Some(msg) = socket.next().await {
            let Ok(Message::Text(msg)) = msg else {
                continue;
            };
            let Ok(msgs) = serde_json::from_str::<Vec<AgentLogMessage>>(&msg) else {
                continue;
            };

            for msg in msgs {
                match msg {
                    AgentLogMessage::Log {
                        step_id,
                        kind,
                        content,
                    } => {
                        for line in content.trim().lines() {
                            info!(
                                "{} {step_id} {}",
                                match kind {
                                    api_types::AgentLogKind::Stdout => "[STDOUT]",
                                    api_types::AgentLogKind::Stderr => "[STDERR]",
                                },
                                line
                            );
                        }
                    }
                }
            }
        }

        debug!("ws closed");
    })
}
