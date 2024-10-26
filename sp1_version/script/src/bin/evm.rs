use script::types;

use aligned_sdk::core::types::{
    AlignedVerificationData, Network, PriceEstimate, ProvingSystemId, VerificationData,
};
use aligned_sdk::sdk::{deposit_to_aligned, estimate_fee};
use aligned_sdk::sdk::{get_next_nonce, submit_and_wait_verification};
use clap::Parser;
use dialoguer::Confirm;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, Bytes, H160, U256};
use reqwest;
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1Stdin};
use std::io;

abigen!(JackRampContract, "JackRampContract.json",);

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZKTRANSFER_ELF: &[u8] =
    include_bytes!("../../../program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    keystore_path: String,
    #[arg(
        short,
        long,
        default_value = "https://ethereum-holesky-rpc.publicnode.com"
    )]
    rpc_url: String,
    #[arg(short, long, default_value = "wss://batcher.alignedlayer.com")]
    batcher_url: String,
    #[arg(short, long, default_value = "holesky")]
    network: Network,
    #[arg(short, long)]
    jackramp_contract_address: H160,
}

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();

    let args = Args::parse();
    let rpc_url = args.rpc_url.clone();

    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .expect("Failed to read keystore password");

    let provider =
        Provider::<Http>::try_from(rpc_url.as_str()).expect("Failed to connect to provider");

    let chain_id = provider
        .get_chainid()
        .await
        .expect("Failed to get chain_id");

    let wallet = LocalWallet::decrypt_keystore(args.keystore_path, &keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(chain_id.as_u64());

    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());

    if Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Do you want to deposit 0.004eth in Aligned ?\nIf you already deposited Ethereum to Aligned before, this is not needed")
        .interact()
        .expect("Failed to read user input") {   

        deposit_to_aligned(U256::from(4000000000000000u128), signer.clone(), args.network).await
        .expect("Failed to pay for proof submission");
    }

    println!("Welcome to the zkTransfer! Input the bank transaction, generate a zkProof, and claim your USD!");
    println!("You will be asked to input Bank, Transaction ID, sender address, and Authorization token.");

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

    println!("{}", "Input your Sender Address?");
    let mut sender_address_answer = String::new();

    io::stdin()
        .read_line(&mut sender_address_answer)
        .expect("Failed to read from stdin");

    println!("{}", "Input your Authorization Token?");
    let mut trx_authorization_answer = String::new();

    io::stdin()
        .read_line(&mut trx_authorization_answer)
        .expect("Failed to read from stdin");

    println!(
        "Answers {}:{}:{}:{}",
        &channel_id_answer.trim(),
        &trx_id_answer.trim(),
        &sender_address_answer.trim(),
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

    let mut proof_data = types::Proof::default();

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<types::Proof>().await {
                Ok(trx_info) => proof_data = trx_info,
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

    let mut stdin = SP1Stdin::new();
    stdin.write(&serde_json::to_string(&proof_data).unwrap());

    let cleaned_address_str = sender_address_answer.trim().trim_start_matches("0x");
    let addr: Address = cleaned_address_str.parse().unwrap();
    stdin.write(addr.as_fixed_bytes());

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ZKTRANSFER_ELF);

    let proof: SP1ProofWithPublicValues = client
        .prove(&pk, stdin)
        .run()
        .expect("failed to generate proof");

    println!("Successfully generated proof!");

    client.verify(&proof, &vk).expect("failed to verify proof");
    println!("Successfully verified proof!");

    // Serialize proof into bincode (format used by sp1)
    let serialized_proof = bincode::serialize(&proof).expect("Failed to serialize proof");

    let verification_data = VerificationData {
        proving_system: ProvingSystemId::SP1,
        proof: serialized_proof,
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(ZKTRANSFER_ELF.to_vec()),
        verification_key: None,
        pub_input: Some(proof.public_values.to_vec()),
    };

    let max_fee = estimate_fee(&rpc_url, PriceEstimate::Instant)
        .await
        .expect("failed to fetch gas price from the blockchain");

    let max_fee_string = ethers::utils::format_units(max_fee, 18).unwrap();

    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("Aligned will use at most {max_fee_string} eth to verify your proof. Do you want to continue?"))
        .interact()
        .expect("Failed to read user input")
    {   return; }

    let nonce = get_next_nonce(&rpc_url, wallet.address(), args.network)
        .await
        .expect("Failed to get next nonce");

    println!("Submitting your proof...");

    let aligned_verification_data = submit_and_wait_verification(
        &args.batcher_url,
        &rpc_url,
        args.network,
        &verification_data,
        max_fee,
        wallet.clone(),
        nonce,
    )
    .await
    .unwrap();

    println!(
        "Proof submitted and verified successfully on batch {}",
        hex::encode(aligned_verification_data.batch_merkle_root)
    );

    println!("Claiming Stablecoins...");

    claim_stablecoin_with_verified_proof(
        &aligned_verification_data,
        signer,
        &args.jackramp_contract_address,
        proof.public_values.as_slice(),
    )
    .await
    .expect("Claiming of Stablecoins failed ...");
}

async fn claim_stablecoin_with_verified_proof(
    aligned_verification_data: &AlignedVerificationData,
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    jackramp_contract_addr: &Address,
    pub_values: &[u8],
) -> anyhow::Result<()> {
    let jr_contract = JackRampContract::new(*jackramp_contract_addr, signer.into());
    let index_in_batch = U256::from(aligned_verification_data.index_in_batch);
    let ver_data_flattened_bytes: Vec<u8> = aligned_verification_data
        .batch_inclusion_proof
        .merkle_path
        .as_slice()
        .iter()
        .flat_map(|array| array.to_vec())
        .collect();

    let merkle_path = Bytes::from(ver_data_flattened_bytes);

    let receipt = jr_contract
        .fill_offramp(
            aligned_verification_data
                .verification_data_commitment
                .proof_commitment,
            aligned_verification_data
                .verification_data_commitment
                .pub_input_commitment,
            aligned_verification_data
                .verification_data_commitment
                .proving_system_aux_data_commitment,
            aligned_verification_data
                .verification_data_commitment
                .proof_generator_addr,
            aligned_verification_data.batch_merkle_root,
            merkle_path,
            index_in_batch,
            Bytes::from(pub_values.to_vec()),
        )
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send tx {}", e))?
        .await
        .map_err(|e| anyhow::anyhow!("Failed to submit tx {}", e))?;

    match receipt {
        Some(receipt) => {
            println!(
                "Stablecoin claimed successfully. Transaction hash: {:x}",
                receipt.transaction_hash
            );
            Ok(())
        }
        None => {
            anyhow::bail!("Failed to claim stablecoin: no receipt");
        }
    }
}
