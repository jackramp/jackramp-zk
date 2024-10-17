#![no_main]
use serde::{Deserialize, Serialize};

sp1_zkvm::entrypoint!(main);

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub data: [TransferData; 1],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferData {
    pub id: String,
    pub bank: String,
    pub to: String,
    pub transfer_date: String,
    pub amount: u64,
}

pub fn main() {
    let data = sp1_zkvm::io::read::<String>();

    let trx_data: Data = serde_json::from_str(&data).unwrap();

    let bank = sp1_zkvm::io::read::<String>();
    let to = sp1_zkvm::io::read::<String>();
    let amount = sp1_zkvm::io::read::<u64>();

    if trx_data.data[0].bank != bank || trx_data.data[0].to != to || trx_data.data[0].amount != amount 
    {
        panic!("Transaction do not match");
    }

    sp1_zkvm::io::commit(&trx_data);
}