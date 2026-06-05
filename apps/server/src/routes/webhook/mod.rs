use poem::{EndpointExt, Route, endpoint::BoxEndpoint};

mod forgejo;

pub fn routes() -> BoxEndpoint<'static> {
    Route::new()
        .nest("/forgejo", forgejo::routes())
        .boxed()
}
