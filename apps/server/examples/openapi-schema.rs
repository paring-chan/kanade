use std::{fs::File, io::Write};

use poem_openapi::OpenApiService;

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
}
