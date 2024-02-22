use axum::{
    routing::{get, post},
    Router,
};
mod handlers;
mod queries;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/clientes/:id/transacoes",
            post(handlers::post_client_transaction_handler::post_client_transaction),
        )
        .route(
            "/clientes/:id/extrato",
            get(handlers::get_client_statement_handler::get_client_statement),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
