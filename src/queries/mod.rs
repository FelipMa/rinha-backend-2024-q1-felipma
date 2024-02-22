pub mod make_client_extract;
pub mod process_transaction;

use sqlx::postgres::PgPoolOptions;

pub async fn get_connection_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    //let connection_string = "postgres://dbuser:dbpassword@localhost:5432/dbname";
    
    //Fábio Akita, se você estiver lendo isso, manda um salve pra mim no próximo vídeo

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    return Ok(pool);
}

pub struct Client {
    pub limit: i32,
    pub balance: i32,
}
