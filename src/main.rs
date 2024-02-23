use axum::{
    routing::{get, post},
    Router,
};
mod handlers;
mod queries;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&connection_string)
        .await
        .expect("Failed to create connection pool");

    let app = Router::new()
        .route(
            "/clientes/:id/transacoes",
            post(handlers::post_client_transaction_handler::post_client_transaction),
        )
        .route(
            "/clientes/:id/extrato",
            get(handlers::get_client_statement_handler::get_client_statement),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
