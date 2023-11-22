use std::env;
use std::net::SocketAddr;
use axum::{
    body::Body,
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router
};
use axum::extract::{Path, Query};
use dotenvy::dotenv;
use crate::db;
use crate::models::NewAdjustmentType;

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
        .route("/adjustment-types", get(list_adjustment_types))
        .route("/adjustment-types", post(create_adjustment_type))
        .route("/adjustment-types/:id", get(get_adjustment_type))
        .route("/adjustment-types/:id", delete(delete_adjustment_type))
        .route("/adjustments", get(list_adjustments))
}

// Handler for the main API endpoint. Returns the version of the API as a JSON object.
async fn index() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    let response = Response::new(Body::from(format!("{{\"version\": \"{}\"}}", version)));
    (StatusCode::OK, response)
}

// GET handler: lists the available adjustment types.
async fn list_adjustment_types() -> impl IntoResponse {
    let adjustment_types = db::get_adjustment_types(None);
    let response = Response::new(Body::from(serde_json::to_string(&adjustment_types).unwrap()));
    (StatusCode::OK, response)
}

// GET handler: shows the adjustment type with the given ID.
async fn get_adjustment_type(Path(id): Path<u64>) -> impl IntoResponse {
    let adjustment_type = db::get_adjustment_type(id);
    // Return a 404 if the adjustment type does not exist.
    match adjustment_type {
        Some(adjustment_type) => {
            let response = Response::new(Body::from(serde_json::to_string(&adjustment_type).unwrap()));
            (StatusCode::OK, response)
        }
        None => {
            let response = Response::new(Body::from(format!("{{\"error\": \"Adjustment type with ID {} not found\"}}", id)));
            (StatusCode::NOT_FOUND, response)
        }
    }
}

// POST handler: creates a new adjustment type.
async fn create_adjustment_type(Json(payload): Json<NewAdjustmentType>) -> impl IntoResponse {
    let rows_inserted = db::add_adjustment_type(payload.description, payload.adjustment);
    // Respond with the number of inserted rows.
    let response = Response::new(Body::from(format!("{{\"inserted\": \"{}\"}}", rows_inserted)));
    (StatusCode::CREATED, response)
}

// DELETE handler: deletes the adjustment type with the given ID.
async fn delete_adjustment_type(Path(id): Path<u64>) -> impl IntoResponse {
    // Return a 404 if the adjustment type does not exist.
    let adjustment_type = db::get_adjustment_type(id);
    match adjustment_type {
        None => {
            let response = Response::new(Body::from(format!("{{\"error\": \"Adjustment type with ID {} not found\"}}", id)));
            return (StatusCode::NOT_FOUND, response);
        }
        _ => {}
    }
    let result = db::delete_adjustment_type(id);
    match result {
        Ok(rows_deleted) => {
            // Respond with the number of deleted rows.
            let response = Response::new(Body::from(format!("{{\"deleted\": \"{}\"}}", rows_deleted)));
            (StatusCode::OK, response)
        }
        Err(e) => {
            // Respond with an error message.
            let response = Response::new(Body::from(format!("{{\"error\": \"{}\"}}", e)));
            (StatusCode::BAD_REQUEST, response)
        }
    }
}

// GET handler: lists the available adjustments, optionally filtered by adjustment type and limit.
async fn list_adjustments(Query(filter): Query<db::AdjustmentQueryFilter>) -> impl IntoResponse {
    let adjustments = db::get_adjustments(filter);
    let response = Response::new(Body::from(serde_json::to_string(&adjustments).unwrap()));
    (StatusCode::OK, response)
}