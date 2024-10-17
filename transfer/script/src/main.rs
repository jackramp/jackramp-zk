#![feature(slice_flatten)]

mod types;

use std::io;
use clap::Parser;
use ethers::prelude::*;
use reqwest;
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1Stdin};

abigen!(VerifierContract, "VerifierContract.json",);

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() {
    println!("Welcome to the zkTransfer! Input the bank transaction, generate a zkProof, and claim your USD!");

    println!("You will be asked to input Bank, Transaction ID, and Authorization token.");

    println!("{}", "What is your Bank ID?");
    let mut bank_id_answer = String::new();

    io::stdin()
        .read_line(&mut bank_id_answer)
        .expect("Failed to read from stdin");

    println!("{}", "Input your Transaction ID?");
    let mut trx_id_answer = String::new();

    io::stdin()
        .read_line(&mut trx_id_answer)
        .expect("Failed to read from stdin");

    println!("{}", "Input your Authorization Token?");
    let mut trx_authorization_answer = String::new();

    io::stdin()
        .read_line(&mut trx_authorization_answer)
        .expect("Failed to read from stdin");

    println!(
        "Answers {}:{}:{}",
        &bank_id_answer.trim(),
        &trx_id_answer.trim(),
        &trx_authorization_answer.trim()
    );

    let client = reqwest::Client::new();

    let request_body = types::TransferRequest {
        id: String::from(trx_id_answer.trim()),
        bank: String::from(bank_id_answer.trim()),
    };

    let source_url: String = String::from("https://mock.blocknaut.xyz/generateTransferProof");
    let response = client
        .post(source_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", &trx_authorization_answer.trim()))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    let mut proof_data = types::TransferResponse::default();

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<types::TransferResponse>().await {
                Ok(trx_info) => proof_data = trx_info,
                Err(error) => println!("Hm, the response didn't match the shape we expected. {}", error),
            };
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            panic!("Need to grab a new token");
        }
        other => {
            panic!("Uh oh! Something unexpected happened: {:?}", other);
        }
    };

    println!("Result. {:?}", &proof_data);

    let params: types::Parameters = serde_json::from_str(&proof_data.claim_data.parameters).unwrap();
    let trx_data: types::Data = serde_json::from_str(&params.response_matches[0].value_resp).unwrap();

    println!("Parameters. {:?}", &trx_data.data[0]);

    println!("Generating Proof ");

    let mut stdin = SP1Stdin::new();
    stdin.write(&params.response_matches[0].value_resp);
    stdin.write(&trx_data.data[0].bank);
    stdin.write(&trx_data.data[0].to);
    stdin.write(&trx_data.data[0].amount);


    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);

    let Ok(proof) = client.prove(&pk, stdin).run() else {
        println!("Incorrect answers!");
        return;
    };

    client.verify(&proof, &vk).expect("verification failed");

    // Test a round trip of proof serialization and deserialization.
    proof.save("proof-with-io.bin").expect("saving proof failed");
    let deserialized_proof =
        SP1ProofWithPublicValues::load("proof-with-io.bin").expect("loading proof failed");

    // Verify the deserialized proof.
    client.verify(&deserialized_proof, &vk).expect("verification failed");

    println!("successfully generated and verified proof for the program!")
}

