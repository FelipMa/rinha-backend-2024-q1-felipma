use crate::queries::{Saldo, Transaction};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::Row;

pub enum StatementError {
    ClientNotFound,
    DatabaseError,
}

pub async fn make_client_extract(
    client_id: i32,
    pool: &sqlx::PgPool,
) -> Result<(Saldo, Vec<Transaction>), StatementError> {
    let mut db_transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Err(StatementError::DatabaseError),
    };

    let client_row =
        match match sqlx::query("SELECT *, NOW() as data_extrato FROM clients WHERE id = $1")
            .bind(&client_id)
            .fetch_optional(&mut *db_transaction)
            .await
        {
            Ok(opt) => opt,
            Err(_) => {
                return Err(StatementError::DatabaseError);
            }
        } {
            Some(row) => row,
            None => {
                db_transaction.commit().await.unwrap();
                return Err(StatementError::ClientNotFound);
            }
        };

    let transaction_rows = match sqlx::query(
        "SELECT * FROM transactions WHERE client_id = $1 ORDER BY id DESC LIMIT 10",
    )
    .bind(&client_id)
    .fetch_all(&mut *db_transaction)
    .await
    {
        Ok(rows) => rows,
        Err(_) => {
            return Err(StatementError::DatabaseError);
        }
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

    match db_transaction.commit().await {
        Ok(_) => {}
        Err(_) => return Err(StatementError::DatabaseError),
    }

    Ok((
        Saldo {
            total: client_row.get("balance"),
            data_extrato: client_row
                .get::<DateTime<Utc>, _>("data_extrato")
                .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            limite: client_row.get("limit"),
        },
        transactions,
    ))
}
