mod api;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:7437";
    let url = "http://localhost:7437";

    let app = Router::new()
        .route("/api/journal", get(api::get_journal).post(api::post_journal))
        .route("/api/journal/:id", delete(api::delete_journal))
        .route("/api/sleep", get(api::get_sleep).post(api::post_sleep))
        .route("/api/tasks", get(api::get_tasks).post(api::post_task))
        .route("/api/tasks/:id", patch(api::patch_task).delete(api::delete_task))
        .route("/api/reports", get(api::get_reports))
        .route("/api/reports/generate", post(api::generate_report))
        .route("/api/reports/:id", get(api::get_report_by_id).delete(api::delete_report))
        .layer(tower_http::cors::CorsLayer::permissive());

    println!("Canopus API → {}", url);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
