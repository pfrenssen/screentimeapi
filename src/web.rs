use crate::db;
use crate::models::{NewAdjustment, NewAdjustmentType, NewTimeEntry};
use axum::extract::{Path, Query, State};
use axum::{
    body::Body,
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use dotenvy::dotenv;
use r2d2::Pool;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    db_pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl AppState {
    pub fn new(db_pool: Pool<ConnectionManager<MysqlConnection>>) -> Self {
        Self { db_pool }
    }
}

pub async fn serve() {
    dotenv().ok();
    let address = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");
    let port = env::var("SERVER_PORT").expect("SERVER_PORT must be set");
    let socket_address: SocketAddr = format!("{address}:{port}")
        .parse()
        .expect("Unable to create a valid socket address.");

    let app = get_app();
    let listener = TcpListener::bind(&socket_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Returns the app routes.
fn get_app() -> Router {
    let db_pool = db::get_connection_pool();
    let app_state = AppState::new(db_pool);

    Router::new()
        .route("/", get(index))
        .route("/adjustment-types", get(list_adjustment_types))
        .route("/adjustment-types", post(create_adjustment_type))
        .route("/adjustment-types/:id", get(get_adjustment_type))
        .route("/adjustment-types/:id", delete(delete_adjustment_type))
        .route("/adjustments", get(list_adjustments))
        .route("/adjustments", post(create_adjustment))
        .route("/time", get(get_adjusted_time))
        .route("/time-entries", get(list_time_entries))
        .route("/time-entries", post(create_time_entry))
        .route("/time-entries/:id", get(get_time_entry))
        .route("/time-entries/:id", delete(delete_time_entry))
        .with_state(app_state)
}

// Handler for the main API endpoint. Returns the version of the API as a JSON object.
async fn index() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    let response = Response::new(Body::from(format!("{{\"version\": \"{version}\"}}")));
    (StatusCode::OK, response)
}

// GET handler: lists the available adjustment types.
async fn list_adjustment_types(State(state): State<AppState>) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let adjustment_types = db::get_adjustment_types(connection, None);
    let response = Response::new(Body::from(
        serde_json::to_string(&adjustment_types).unwrap(),
    ));
    (StatusCode::OK, response)
}

// GET handler: shows the adjustment type with the given ID.
async fn get_adjustment_type(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let adjustment_type = db::get_adjustment_type(connection, id);

    if let Some(adjustment_type) = adjustment_type {
        let response = Response::new(Body::from(serde_json::to_string(&adjustment_type).unwrap()));
        (StatusCode::OK, response)
    } else {
        let response = Response::new(Body::from(format!(
            "{{\"error\": \"Adjustment type with ID {id} not found\"}}"
        )));
        (StatusCode::NOT_FOUND, response)
    }
}

// POST handler: creates a new adjustment type.
async fn create_adjustment_type(
    State(state): State<AppState>,
    Json(payload): Json<NewAdjustmentType>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let rows_inserted =
        db::add_adjustment_type(connection, payload.description, payload.adjustment);
    // Respond with the number of inserted rows.
    let response = Response::new(Body::from(format!("{{\"inserted\": \"{rows_inserted}\"}}")));
    (StatusCode::CREATED, response)
}

// DELETE handler: deletes the adjustment type with the given ID.
async fn delete_adjustment_type(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    // Return a 404 if the adjustment type does not exist.
    let adjustment_type = db::get_adjustment_type(connection, id);
    if adjustment_type.is_none() {
        let response = Response::new(Body::from(format!(
            "{{\"error\": \"Adjustment type with ID {id} not found\"}}"
        )));
        return (StatusCode::NOT_FOUND, response);
    }

    let result = db::delete_adjustment_type(connection, id);
    match result {
        Ok(rows_deleted) => {
            // Respond with the number of deleted rows.
            let response =
                Response::new(Body::from(format!("{{\"deleted\": \"{rows_deleted}\"}}")));
            (StatusCode::OK, response)
        }
        Err(e) => {
            // Respond with an error message.
            let response = Response::new(Body::from(format!("{{\"error\": \"{e}\"}}")));
            (StatusCode::BAD_REQUEST, response)
        }
    }
}

// GET handler: lists the available adjustments, optionally filtered by adjustment type and limit.
async fn list_adjustments(
    State(state): State<AppState>,
    Query(filter): Query<db::AdjustmentQueryFilter>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let adjustments = db::get_adjustments(connection, &filter);
    let response = Response::new(Body::from(serde_json::to_string(&adjustments).unwrap()));
    (StatusCode::OK, response)
}

// POST handler: creates a new adjustment.
async fn create_adjustment(
    State(state): State<AppState>,
    Json(payload): Json<NewAdjustment>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let adjustment_type = db::get_adjustment_type(connection, payload.adjustment_type_id);
    if let Some(adjustment_type) = adjustment_type {
        let rows_inserted =
            db::add_adjustment(connection, &adjustment_type, &payload.comment, &None);
        // Respond with the number of inserted rows.
        let response = Response::new(Body::from(format!("{{\"inserted\": \"{rows_inserted}\"}}")));
        (StatusCode::CREATED, response)
    } else {
        // Return a 404 if the adjustment type does not exist.
        let response = Response::new(Body::from(format!(
            "{{\"error\": \"Adjustment type with ID {} not found\"}}",
            payload.adjustment_type_id
        )));
        (StatusCode::NOT_FOUND, response)
    }
}

// GET handler: returns the current time, adjusted by the available adjustments.
async fn get_adjusted_time(State(state): State<AppState>) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let adjusted_time = db::get_adjusted_time(connection);
    let formatted_time = format!("{:01}:{:02}", adjusted_time / 60, adjusted_time % 60);
    let response = Response::new(Body::from(format!(
        "{{\"time\":{adjusted_time},\"formatted_time\":\"{formatted_time}\"}}"
    )));
    (StatusCode::OK, response)
}

// GET handler: lists the available time entries.
async fn list_time_entries(State(state): State<AppState>) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let time_entries = db::get_time_entries(connection, None);
    let response = Response::new(Body::from(serde_json::to_string(&time_entries).unwrap()));
    (StatusCode::OK, response)
}

// POST handler: creates a new time entry.
async fn create_time_entry(
    State(state): State<AppState>,
    Json(payload): Json<NewTimeEntry>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let rows_inserted = db::add_time_entry(connection, payload.time, payload.created);
    // Respond with the number of inserted rows.
    let response = Response::new(Body::from(format!("{{\"inserted\": \"{rows_inserted}\"}}")));
    (StatusCode::CREATED, response)
}

// GET handler: shows the time entry with the given ID.
async fn get_time_entry(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    let time_entry = db::get_time_entry(connection, id);

    if let Some(time_entry) = time_entry {
        let response = Response::new(Body::from(serde_json::to_string(&time_entry).unwrap()));
        (StatusCode::OK, response)
    } else {
        let response = Response::new(Body::from(format!(
            "{{\"error\": \"Time entry with ID {id} not found\"}}"
        )));
        (StatusCode::NOT_FOUND, response)
    }
}

/// DELETE handler: deletes the time entry with the given ID.
async fn delete_time_entry(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let pool = &state.db_pool;
    let connection = &mut pool.get().unwrap();
    // Return a 404 if the time entry does not exist.
    let time_entry = db::get_time_entry(connection, id);
    if time_entry.is_none() {
        let response = Response::new(Body::from(format!(
            "{{\"error\": \"Time entry with ID {id} not found\"}}"
        )));
        return (StatusCode::NOT_FOUND, response);
    }

    let rows_deleted = db::delete_time_entry(connection, id);
    let response = Response::new(Body::from(format!("{{\"deleted\": \"{rows_deleted}\"}}")));
    (StatusCode::OK, response)
}
