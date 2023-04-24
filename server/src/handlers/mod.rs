use crate::http_responses::HttpJsonResponse;
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str;

#[derive(Deserialize)]
struct Param {
    param: String,
}

#[derive(Deserialize, Serialize)]
struct CommandResponse {
    stdout: String,
    stderr: String,
    exit_status: i32,
}

#[get("/")]
pub async fn index() -> HttpResponse {
    HttpJsonResponse::ok("Playground API")
}

#[get("/skaffold")]
pub async fn skaffold(param: web::Query<Param>) -> HttpResponse {
    let mut sh = Command::new("sh");
    let output = sh
        .arg("-c")
        .arg(format!("skaffold {}", param.param))
        .output();

    if output.is_err() {
        return HttpJsonResponse::internal_server_error(output.unwrap_err().to_string());
    } else {
        let stdout = str::from_utf8(&*output.as_ref().unwrap().stdout)
            .unwrap()
            .to_string();
        let stderr = str::from_utf8(&*output.as_ref().unwrap().stderr)
            .unwrap()
            .to_string();
        let exit_status = output.unwrap().status.code().unwrap();

        HttpJsonResponse::ok(CommandResponse {
            stdout,
            stderr,
            exit_status,
        })
    }
}
