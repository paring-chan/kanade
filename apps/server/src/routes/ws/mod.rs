use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use poem::{
    EndpointExt, IntoResponse, Route,
    endpoint::BoxEndpoint,
    handler,
    web::{
        Data,
        websocket::{Message, WebSocket},
    },
};

use crate::realtime::Realtime;

pub fn routes() -> BoxEndpoint<'static> {
    Route::new().at("/events", events_ws).boxed()
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
