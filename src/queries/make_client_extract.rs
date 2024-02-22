use crate::queries::{get_connection_pool, Client};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::Row;

pub enum StatementError {
    ClientNotFound,
    DatabaseError(String),
}

#[derive(serde::Serialize)]
pub struct Transaction {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    pub realizada_em: String,
}

pub async fn make_client_extract(
    client_id: i32,
) -> Result<(Client, Vec<Transaction>), StatementError> {
    let pool = match get_connection_pool().await {
        Ok(pool) => pool,
        Err(err) => return Err(StatementError::DatabaseError(err.to_string())),
    };

    let mut db_transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(err) => return Err(StatementError::DatabaseError(err.to_string())),
    };

    let client_row = match match sqlx::query("SELECT * FROM clients WHERE id = $1")
        .bind(&client_id)
        .fetch_optional(&mut *db_transaction)
        .await
    {
        Ok(opt) => opt,
        Err(err) => return Err(StatementError::DatabaseError(err.to_string())),
    } {
        Some(row) => row,
        None => return Err(StatementError::ClientNotFound),
    };

    let transaction_rows = match sqlx::query(
        "SELECT * FROM transactions WHERE client_id = $1 ORDER BY id DESC LIMIT 10",
    )
    .bind(&client_id)
    .fetch_all(&mut *db_transaction)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return Err(StatementError::DatabaseError(err.to_string())),
    };

    let mut transactions = Vec::new();

    for row in transaction_rows {
        transactions.push(Transaction {
            valor: row.get("value"),
            tipo: row.get("type"),
            descricao: row.get("description"),
            realizada_em: DateTime::<Utc>::from_naive_utc_and_offset(
                row.get::<NaiveDateTime, _>("date"),
                Utc,
            )
            .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        });
    }

    Ok((
        Client {
            balance: client_row.get("balance"),
            limit: client_row.get("limit"),
        },
        transactions,
    ))
}