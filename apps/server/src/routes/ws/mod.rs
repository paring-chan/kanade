use std::sync::Arc;

use api_types::AgentLogMessage;
use futures_util::StreamExt;
use poem::{
    EndpointExt, IntoResponse, Route,
    endpoint::BoxEndpoint,
    get, handler,
    web::{
        Data,
        websocket::{Message, WebSocket},
    },
};

use crate::realtime::{
    Realtime,
    types::{LogEntry, LogMessage},
};

pub fn routes() -> BoxEndpoint<'static> {
    Route::new().at("/agent", get(handle_agent)).boxed()
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
                        job_id,
                        step_id,
                        content,
                    } => {
                        _ = realtime
                            .publish_log(&LogMessage {
                                job_id: job_id,
                                entry: LogEntry { content, step_id },
                            })
                            .await;
                    }
                }
            }
        }

        debug!("ws closed");
    })
}
