use crate::queries;
use axum::{
    body::Body,
    extract::{Json, Path},
    http::StatusCode,
    response::Response,
};
use chrono::Utc;
use serde_json::json;

#[derive(serde::Deserialize)]
pub struct CreateTransactionPayload {
    valor: i32,
    tipo: char,
    descricao: String,
}

pub async fn post_client_transaction(
    Path(client_id): Path<String>,
    payload: Json<CreateTransactionPayload>,
) -> Response {
    if payload.tipo != 'd' && payload.tipo != 'c' {
        return Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(Body::empty())
            .unwrap();
    }

    let parsed_client_id = match client_id.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(Body::empty())
                .unwrap()
        }
    };

    let now = Utc::now();

    match queries::process_transaction::process_transaction(
        parsed_client_id,
        payload.valor,
        payload.tipo.to_string(),
        payload.descricao.clone(),
        now,
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
            queries::process_transaction::TransactionError::DatabaseError(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(err.into())
                    .unwrap()
            }
        },
    };
}
