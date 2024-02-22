use crate::queries::{get_connection_pool, Client};
use chrono::{DateTime, Utc};
use sqlx::Row;

pub enum TransactionError {
    InsufficientFunds,
    ClientNotFound,
    DatabaseError(String),
}

pub async fn process_transaction(
    client_id: i32,
    transaction_value: i32,
    transaction_type: String,
    transaction_description: String,
    transaction_date: DateTime<Utc>,
) -> Result<Client, TransactionError> {
    let pool = match get_connection_pool().await {
        Ok(pool) => pool,
        Err(err) => return Err(TransactionError::DatabaseError(err.to_string())),
    };

    let mut db_transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(err) => return Err(TransactionError::DatabaseError(err.to_string())),
    };

    let client_row = match match sqlx::query("SELECT * FROM clients WHERE id = $1")
        .bind(&client_id)
        .fetch_optional(&mut *db_transaction)
        .await
    {
        Ok(opt) => opt,
        Err(err) => return Err(TransactionError::DatabaseError(err.to_string())),
    } {
        Some(row) => row,
        None => return Err(TransactionError::ClientNotFound),
    };

    if transaction_type == "d"
        && (client_row.get::<i32, _>("balance") - transaction_value)
            < -1 * client_row.get::<i32, _>("limit")
    {
        return Err(TransactionError::InsufficientFunds);
    }

    match sqlx::query(
        "INSERT INTO transactions (client_id, value, type, description, date) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&client_id)
    .bind(&transaction_value)
    .bind(&transaction_type)
    .bind(&transaction_description)
    .bind(&transaction_date)
    .execute(&mut *db_transaction)
    .await
    {
        Ok(_) => {}
        Err(err) => return Err(TransactionError::DatabaseError(err.to_string())),
    };

    let operation = if transaction_type == "d" {
        -1 * transaction_value
    } else {
        transaction_value
    };

    let updated_client_row =
        match sqlx::query("UPDATE clients SET balance = balance + $1 WHERE id = $2 RETURNING *")
            .bind(&operation)
            .bind(&client_id)
            .fetch_one(&mut *db_transaction)
            .await
        {
            Ok(row) => row,
            Err(err) => return Err(TransactionError::DatabaseError(err.to_string())),
        };

    match db_transaction.commit().await {
        Ok(_) => {}
        Err(err) => return Err(TransactionError::DatabaseError(err.to_string())),
    }

    return Ok(Client {
        limit: updated_client_row.get("limit"),
        balance: updated_client_row.get("balance"),
    });
}
