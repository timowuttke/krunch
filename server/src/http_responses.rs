use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde::Serialize;

pub struct HttpJsonResponse {}

impl HttpJsonResponse {
    pub fn ok<T: Serialize>(body: T) -> HttpResponse {
        HttpResponse::build(StatusCode::OK)
            .content_type("application/json")
            .body(serde_json::json!(body).to_string())
    }

    pub fn bad_request<T: Serialize>(body: T) -> HttpResponse {
        HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type("application/json")
            .body(serde_json::json!({ "error": body }).to_string())
    }

    pub fn invalid_token<T: Serialize>(body: T) -> HttpResponse {
        HttpResponse::build(StatusCode::UNAUTHORIZED)
            .content_type("application/json")
            .body(serde_json::json!({ "error": body }).to_string())
    }

    pub fn internal_server_error<T: Serialize>(body: T) -> HttpResponse {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/json")
            .body(serde_json::json!({ "error": body }).to_string())
    }
}
