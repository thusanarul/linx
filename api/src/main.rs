mod conversion;

use axum::{Router, response::Html, routing::get};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // build our application with a single route
    let app = Router::new().route("/", get(hello));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let server_handle = tokio::spawn(async move {
        tracing::info!("Starting server...");
        axum::serve(listener, app).await.unwrap();
    });

    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("Shutting down server...")
        }
        Err(_) => {
            tracing::error!("Unable to listen for shutdown signal...")
        }
    }

    server_handle.abort();

    Ok(())
}

async fn hello() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
