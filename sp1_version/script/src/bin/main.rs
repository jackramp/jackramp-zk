use script::types;

#[warn(unused_imports)]
use alloy_sol_types::SolType;
use clap::Parser;
use reqwest;
use sp1_sdk::{ProverClient, SP1Stdin};
use std::io;
use zktransfer_lib::PublicValuesStruct;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZKTRANSFER_ELF: &[u8] =
    include_bytes!("../../../program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,
}

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();

    let args = Args::parse();
    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

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

    let client = reqwest::Client::new();

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
        .await
        .unwrap();

    let mut proof = types::Proof::default();

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<types::Proof>().await {
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

    let wallet: LocalWallet = LocalWallet::new(&mut rand::thread_rng());
    let random_address: Address = wallet.address();

    let mut stdin = SP1Stdin::new();
    stdin.write(&serde_json::to_string(&proof).unwrap());
    stdin.write(random_address.as_fixed_bytes());

    let client = ProverClient::new();

    if args.execute {
        let (output, report) = client.execute(ZKTRANSFER_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let decoded = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();
        println!("hash channel id: {:?}", decoded.offrampRequestParams.hashedChannelId);
        println!("hash channel account: {:?}", decoded.offrampRequestParams.hashedChannelAccount);
        println!("amount: {}", decoded.offrampRequestParams.amount);
        println!("rw amount: {}", decoded.offrampRequestParams.amountRealWorld);

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        let (pk, vk) = client.setup(ZKTRANSFER_ELF);

        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
