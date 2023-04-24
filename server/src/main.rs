use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpServer};
use crate::handlers::{index, skaffold};

mod handlers;
mod http_responses;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // log level is controlled by RUST_LOG env variable, e.g. RUST_LOG="warn"
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%s %r %{User-Agent}i %a"))
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost")
                    .allowed_origin("https://k8s.local")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .service(index)
            .service(skaffold)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
