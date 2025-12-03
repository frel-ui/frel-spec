// HTTP server setup using actix-web

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};

use crate::api;
use crate::state::SharedState;

/// Create the HTTP server (does not start it - caller must await)
pub fn run_server(state: SharedState, port: u16) -> std::io::Result<Server> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/status", web::get().to(api::get_status))
            .route("/modules", web::get().to(api::get_modules))
            .route("/diagnostics", web::get().to(api::get_all_diagnostics))
            .route("/diagnostics/{module:.*}", web::get().to(api::get_module_diagnostics))
            .route("/ast/{module:.*}", web::get().to(api::get_module_ast))
            .route("/generated/{module:.*}", web::get().to(api::get_module_generated))
            .route("/scope/{module:.*}", web::get().to(api::get_module_scope))
            .route("/source/{path:.*}", web::get().to(api::get_source))
            .route("/notify", web::post().to(api::post_notify))
            .route("/write", web::post().to(api::post_write))
            .route("/events", web::get().to(api::get_events))
            // Expectations endpoints (compiler dev mode)
            .route("/expectations/{module:.*}/save", web::post().to(api::save_expectations))
            .route("/expectations/{module:.*}", web::get().to(api::get_expectations))
            .route("/compare/{module:.*}", web::get().to(api::compare_expectations))
    })
    .disable_signals() // We handle signals manually
    .bind(("0.0.0.0", port))?
    .run();

    Ok(server)
}
