use std::{sync::Arc, time::Duration};

use poem::{
    EndpointExt, Route,
    endpoint::BoxEndpoint,
    get, handler,
    web::{
        Data, Path,
        sse::{Event, SSE},
    },
};
use uuid::Uuid;

use crate::realtime::{Realtime, types::LogMessage};

pub fn routes() -> BoxEndpoint<'static> {
    Route::new()
        .at("/events", get(events_sse))
        .at("/logs/:job_id", get(log_ws))
        .boxed()
}

#[handler]
async fn log_ws(Data(realtime): Data<&Arc<Realtime>>, Path(job_id_param): Path<Uuid>) -> SSE {
    let realtime = realtime.clone();

    let mut receiver = realtime.log_stream.subscribe();

    SSE::new(async_stream::stream! {
        while let Ok(msg) = receiver.recv().await {
            match msg {
                LogMessage { job_id, entry } => {
                    if job_id != job_id_param {
                        continue;
                    }
                    if let Ok(data) = serde_json::to_string(&entry)
                    {
                        yield Event::message(data);
                    }
                }
            }
        }
    })
    .keep_alive(Duration::from_secs(5))
}

#[handler]
async fn events_sse(Data(realtime): Data<&Arc<Realtime>>) -> SSE {
    let mut receiver = realtime.event_stream.subscribe();
    SSE::new(async_stream::stream! {
         while let Ok(msg) = receiver.recv().await {
            if let Ok(data) = serde_json::to_string(&msg)
            {
                yield Event::message(data);
            }
        }

    })
    .keep_alive(Duration::from_secs(5))
}
