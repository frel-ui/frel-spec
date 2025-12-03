// HTTP server setup using actix-web

use actix_web::{web, App, HttpServer};

use crate::api;
use crate::state::SharedState;

/// Run the HTTP server
pub async fn run_server(state: SharedState, port: u16) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/status", web::get().to(api::get_status))
            .route("/modules", web::get().to(api::get_modules))
            .route("/diagnostics", web::get().to(api::get_all_diagnostics))
            .route("/diagnostics/{module:.*}", web::get().to(api::get_module_diagnostics))
            .route("/ast/{module:.*}", web::get().to(api::get_module_ast))
            .route("/generated/{module:.*}", web::get().to(api::get_module_generated))
            .route("/notify", web::post().to(api::post_notify))
            .route("/events", web::get().to(api::get_events))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
