use crate::queries;
use axum::{
    body::Body,
    extract::{Json, Path, State},
    http::StatusCode,
    response::Response,
};
use serde_json::json;

#[derive(serde::Deserialize)]
pub struct CreateTransactionPayload {
    valor: i32,
    tipo: String,
    descricao: String,
}

pub async fn post_client_transaction(
    State(pool): State<sqlx::PgPool>,
    Path(client_id): Path<String>,
    payload: Json<CreateTransactionPayload>,
) -> Response {
    let parsed_client_id = match client_id.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(Body::empty())
                .unwrap()
        }
    };

    if payload.tipo != "d" && payload.tipo != "c" {
        return Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(Body::empty())
            .unwrap();
    }

    if payload.descricao.len() < 1 || payload.descricao.len() > 10 {
        return Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(Body::empty())
            .unwrap();
    }

    match queries::process_transaction::process_transaction(
        parsed_client_id,
        payload.valor,
        payload.tipo.clone(),
        payload.descricao.clone(),
        &pool,
    )
    .await
    {
        Ok(client) => {
            return Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(
                    serde_json::to_string(&json!({
                        "limite": client.limit,
                        "saldo": client.balance,
                    }))
                    .unwrap()
                    .into(),
                )
                .unwrap()
        }
        Err(err) => match err {
            queries::process_transaction::TransactionError::InsufficientFunds => {
                return Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(Body::empty())
                    .unwrap()
            }
            queries::process_transaction::TransactionError::ClientNotFound => {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap()
            }
            queries::process_transaction::TransactionError::DatabaseError => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap();
            }
        },
    };
}
