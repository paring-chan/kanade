use poem::{EndpointExt, Route, endpoint::BoxEndpoint};

pub fn routes() -> BoxEndpoint<'static> {
    Route::new().boxed()
}
