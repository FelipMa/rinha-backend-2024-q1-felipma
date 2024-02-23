pub mod make_client_extract;
pub mod process_transaction;

pub struct Client {
    pub limit: i32,
    pub balance: i32,
}

#[derive(serde::Serialize)]
pub struct Transaction {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    pub realizada_em: String,
}

#[derive(serde::Serialize)]
pub struct Statement {
    pub saldo: Saldo,
    pub ultimas_transacoes: Vec<Transaction>,
}

#[derive(serde::Serialize)]
pub struct Saldo {
    pub total: i32,
    pub data_extrato: String,
    pub limite: i32,
}
