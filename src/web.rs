use std::env;
use std::net::SocketAddr;
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router
};
use dotenvy::dotenv;

pub async fn serve() {
    dotenv().ok();
    let address = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");
    let port = env::var("SERVER_PORT").expect("SERVER_PORT must be set");
    let socket_address: SocketAddr = format!("{}:{}", address, port).parse().expect("Unable to create a valid socket address.");

    let app = get_app();
    axum::Server::bind(&socket_address)
        .serve(app.await.into_make_service())
        .await
        .unwrap();
}

// Returns the app routes.
async fn get_app() -> Router {
    Router::new()
        .route("/", get(index))
}

// Handler for the main API endpoint. Returns the version of the API as a JSON object.
async fn index() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    let response = Response::new(Body::from(format!("{{\"version\": \"{}\"}}", version)));
    (StatusCode::OK, response)
}