use std::{fs::File, io::Write};

use poem_openapi::OpenApiService;
use server::EventMessage;

fn main() {
    File::create("apps/frontend/api-schema.json")
        .unwrap()
        .write_all(
            OpenApiService::new(server::api(), "Kanade API", "1.0")
                .url_prefix("/api/v1")
                .spec()
                .as_bytes(),
        )
        .unwrap();

    let types = specta::Types::default().register::<EventMessage>();
    let data = specta_typescript::Typescript::default()
        .export(&types, specta_serde::PhasesFormat)
        .unwrap();

    File::create("apps/frontend/src/ws-types.d.ts")
        .unwrap()
        .write_all(data.as_bytes())
        .unwrap();
}
