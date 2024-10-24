#![allow(unused)]

mod types;

use zk_rust_io;
use alloy_primitives::{keccak256, Address, Bytes, FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolType};
use hex::FromHex;
use reqwest;
use std::io;

sol! {
    #[derive(Debug)]
    struct CompleteClaimData {
        bytes32 identifier;
        address owner;
        uint32 timestampS;
        uint32 epoch;
    }

    #[derive(Debug)]
    struct SignedClaim {
        CompleteClaimData claim;
        bytes[] signatures;
    }

    #[derive(Debug)]
    struct PublicValuesStruct {
      bytes32 hashedChannelId;
      bytes32 hashedChannelAccount;
      uint256 amount;
      bytes32 hashedClaimInfo;
      SignedClaim signedClaim;
    }
}

fn main() {
    let raw_proof_data: String = zk_rust_io::read();
    let proof: types::Proof = serde_json::from_str(&raw_proof_data).unwrap();
    let parameters: types::Parameters = serde_json::from_str(&proof.claim_info.parameters).unwrap();
    let transaction: types::Data =
        serde_json::from_str(&parameters.response_matches[0].value_resp).unwrap();

    let mut encoded_claim_info: Vec<u8> = Vec::new();
    encoded_claim_info.extend_from_slice(proof.claim_info.provider.as_bytes());
    encoded_claim_info.extend_from_slice(b"\n");
    encoded_claim_info.extend_from_slice(proof.claim_info.parameters.as_bytes());
    encoded_claim_info.extend_from_slice(b"\n");
    encoded_claim_info.extend_from_slice(proof.claim_info.context.as_bytes());

    let hashed_claim_info: B256 = keccak256(encoded_claim_info);
    let hashed_channel_id: B256 = keccak256(&transaction.data[0].bank);
    let hashed_channel_account: B256 = keccak256(&transaction.data[0].to);
    let amount: U256 = U256::from(transaction.data[0].amount);
    let identifier = proof
        .signed_claim
        .claim
        .identifier
        .parse::<FixedBytes<32>>()
        .unwrap();

    let address_str = proof.signed_claim.claim.owner.strip_prefix("0x").unwrap();
    let address_bytes: [u8; 20] = <[u8; 20]>::from_hex(address_str).expect("Invalid hex string");
    let owner = Address::from(address_bytes);

    let signatures = proof
        .signed_claim
        .signatures
        .iter()
        .map(|v| v.as_str().to_string().into())
        .collect::<Vec<Bytes>>();

    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct {
        hashedChannelId: hashed_channel_id,
        hashedChannelAccount: hashed_channel_account,
        amount: amount,
        hashedClaimInfo: hashed_claim_info,
        signedClaim: SignedClaim {
            claim: CompleteClaimData {
                epoch: proof.signed_claim.claim.epoch,
                identifier: identifier,
                owner: owner,
                timestampS: proof.signed_claim.claim.timestamp,
            },
            signatures,
        },
    });

    zk_rust_io::commit(&bytes);
}

fn input() {
    println!("Welcome to the zkTransfer! Input the bank transaction, generate a zkProof, and claim your USD!");
    println!("You will be asked to input Bank, Transaction ID, and Authorization token.");

    println!("{}", "What is your Channel ID?");
    let mut channel_id_answer = String::new();

    io::stdin()
        .read_line(&mut channel_id_answer)
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
        &channel_id_answer.trim(),
        &trx_id_answer.trim(),
        &trx_authorization_answer.trim()
    );

    let client = reqwest::blocking::Client::new();

    let request_body = types::TransferRequest {
        id: String::from(trx_id_answer.trim()),
        bank: String::from(channel_id_answer.trim()),
    };

    let source_url: String = String::from("https://mock.blocknaut.xyz/generateTransferProof");
    let response = client
        .post(source_url)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!("Bearer {}", &trx_authorization_answer.trim()),
        )
        .json(&request_body)
        .send()
        .unwrap();

    let mut proof = types::Proof::default();

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<types::Proof>() {
                Ok(trx_info) => proof = trx_info,
                Err(error) => println!(
                    "Hm, the response didn't match the shape we expected. {}",
                    error
                ),
            };
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            panic!("Need to grab a new token");
        }
        other => {
            panic!("Uh oh! Something unexpected happened: {:?}", other);
        }
    };

    println!("Generating Proof ");

    zk_rust_io::write(&serde_json::to_string(&proof).unwrap());
}

fn output() {
    let output: Vec<u8> = zk_rust_io::out();
    let decoded = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();
    println!("public value: {:?}", decoded);
}
