use poem::{EndpointExt, Route, endpoint::BoxEndpoint};

mod webhook;

pub fn routes() -> BoxEndpoint<'static> {
    Route::new()
        .nest("/webhook", webhook::routes())
        .boxed()
}
