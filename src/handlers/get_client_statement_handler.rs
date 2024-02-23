use crate::queries;
use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};
use chrono::Utc;

pub async fn get_client_statement(
    State(pool): State<sqlx::PgPool>,
    Path(client_id): Path<String>,
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

    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    let (client, transactions) =
        match queries::make_client_extract::make_client_extract(parsed_client_id, &pool).await {
            Ok((client, transactions)) => (client, transactions),
            Err(err) => match err {
                queries::make_client_extract::StatementError::ClientNotFound => {
                    return Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()
                }
                queries::make_client_extract::StatementError::DatabaseError => {
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap()
                }
            },
        };

    #[derive(serde::Serialize)]
    struct Statement {
        saldo: Saldo,
        ultimas_transacoes: Vec<queries::make_client_extract::Transaction>,
    }

    #[derive(serde::Serialize)]
    struct Saldo {
        total: i32,
        data_extrato: String,
        limite: i32,
    }

    let saldo = Saldo {
        total: client.balance,
        data_extrato: now,
        limite: client.limit,
    };

    let statement = Statement {
        saldo,
        ultimas_transacoes: transactions,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&statement).unwrap().into())
        .unwrap()
}
