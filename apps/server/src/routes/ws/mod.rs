use std::sync::Arc;

use api_types::AgentLogMessage;
use futures_util::{SinkExt, StreamExt};
use poem::{
    EndpointExt, IntoResponse, Route,
    endpoint::BoxEndpoint,
    get, handler,
    web::{
        Data, Path,
        websocket::{Message, WebSocket},
    },
};
use uuid::Uuid;

use crate::realtime::{
    Realtime,
    types::{LogEntry, LogMessage},
};

pub fn routes() -> BoxEndpoint<'static> {
    Route::new()
        .at("/events", get(events_ws))
        .at("/agent", get(handle_agent))
        .at("/logs/:job_id", get(log_ws))
        .boxed()
}

#[handler]
async fn log_ws(
    ws: WebSocket,
    Data(realtime): Data<&Arc<Realtime>>,
    Path(job_id_param): Path<Uuid>,
) -> impl IntoResponse {
    let realtime = realtime.clone();

    ws.on_upgrade(move |socket| async move {
        let (mut sink, _) = socket.split();
        let mut receiver = realtime.log_stream.subscribe();

        while let Ok(msg) = receiver.recv().await {
            match msg {
                LogMessage { job_id, entry } => {
                    if job_id != job_id_param {
                        continue;
                    }
                    if let Ok(data) = serde_json::to_string(&entry)
                        && let Err(_) = sink.send(Message::Text(data)).await
                    {
                        break;
                    }
                }
            }
        }
    })
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
