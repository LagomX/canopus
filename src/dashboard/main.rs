mod api;

use axum::{
    http::header,
    response::{Html, IntoResponse},
    routing::{delete, get},
    Router,
};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:7437";
    let url = "http://localhost:7437";

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/style.css", get(serve_css))
        .route("/app.js", get(serve_js))
        .route("/api/journal", get(api::get_journal).post(api::post_journal))
        .route("/api/journal/:id", delete(api::delete_journal))
        .route("/api/sleep", get(api::get_sleep).post(api::post_sleep))
        .layer(tower_http::cors::CorsLayer::permissive());

    println!("Canopus Dashboard → {}", url);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Open browser after a brief yield so the server starts first
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        let _ = open::that("http://localhost:7437");
    });

    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("static/index.html"))
}

async fn serve_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("static/style.css"),
    )
}

async fn serve_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("static/app.js"),
    )
}
