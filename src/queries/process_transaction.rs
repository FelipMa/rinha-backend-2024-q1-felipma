use crate::queries::Client;
use sqlx::Row;

pub enum TransactionError {
    InsufficientFunds,
    ClientNotFound,
    DatabaseError,
}

pub async fn process_transaction(
    client_id: i32,
    transaction_value: i32,
    transaction_type: String,
    transaction_description: String,
    pool: &sqlx::PgPool,
) -> Result<Client, TransactionError> {
    let mut db_transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Err(TransactionError::DatabaseError),
    };

    let client_row = match match sqlx::query("SELECT * FROM clients WHERE id = $1")
        .bind(&client_id)
        .fetch_optional(&mut *db_transaction)
        .await
    {
        Ok(opt) => opt,
        Err(_) => {
            return Err(TransactionError::DatabaseError);
        }
    } {
        Some(row) => row,
        None => {
            db_transaction.commit().await.unwrap();
            return Err(TransactionError::ClientNotFound);
        }
    };

    if transaction_type == "d"
        && (client_row.get::<i32, _>("balance") - transaction_value)
            < -1 * client_row.get::<i32, _>("limit")
    {
        db_transaction.commit().await.unwrap();
        return Err(TransactionError::InsufficientFunds);
    }

    match sqlx::query(
        "INSERT INTO transactions (client_id, value, type, description) VALUES ($1, $2, $3, $4)",
    )
    .bind(&client_id)
    .bind(&transaction_value)
    .bind(&transaction_type)
    .bind(&transaction_description)
    .execute(&mut *db_transaction)
    .await
    {
        Ok(_) => {}
        Err(_) => {
            db_transaction.rollback().await.unwrap();
            return Err(TransactionError::DatabaseError);
        }
    }

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
            Err(_) => {
                db_transaction.rollback().await.unwrap();
                return Err(TransactionError::DatabaseError);
            }
        };

    match db_transaction.commit().await {
        Ok(_) => {}
        Err(_) => return Err(TransactionError::DatabaseError),
    }

    return Ok(Client {
        limit: updated_client_row.get("limit"),
        balance: updated_client_row.get("balance"),
    });
}
