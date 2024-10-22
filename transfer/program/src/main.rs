#![no_main]
sp1_zkvm::entrypoint!(main);

mod types;

use alloy_primitives::{keccak256, Address, Bytes, FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolType};
use hex::FromHex;

sol! {
  #[derive(Debug)]
  struct CompleteClaimData {
      bytes32 identifier;
      address owner;
      uint32 timestampS;
      uint32 epoch;
  }
}

sol! {
  #[derive(Debug)]
  struct SignedClaim {
      CompleteClaimData claim;
      bytes[] signatures;
  }
}

sol! {
  #[derive(Debug)]
  struct PublicValuesStruct {
    bytes32 hashedChannelId;
    bytes32 hashedChannelAccount;
    uint256 amount;
    bytes32 hashedClaimInfo;
    SignedClaim signedClaim;
  }
}

pub fn main() {
    let raw_proof_data = sp1_zkvm::io::read::<String>();
    let proof: types::Proof = serde_json::from_str(&raw_proof_data).unwrap();
    let parameters: types::Parameters = serde_json::from_str(&proof.claim_info.parameters).unwrap();
    let transaction: types::Data = serde_json::from_str(&parameters.response_matches[0].value_resp).unwrap();

    let mut encoded_claim_info: Vec<u8> = Vec::new();
    encoded_claim_info.extend_from_slice(proof.claim_info.provider.as_bytes());
    encoded_claim_info.extend_from_slice(b"\n");
    encoded_claim_info.extend_from_slice(proof.claim_info.parameters.as_bytes());
    encoded_claim_info.extend_from_slice(b"\n");
    encoded_claim_info.extend_from_slice(proof.claim_info.context.as_bytes());

    let hashed_claim_info: B256 = keccak256(encoded_claim_info);
    let hashed_channel_id: B256 =
        keccak256(&transaction.data[0].bank);
    let hashed_channel_account: B256 =
        keccak256(&transaction.data[0].to);
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

    sp1_zkvm::io::commit_slice(&bytes);
}
