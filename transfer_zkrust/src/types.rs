use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Serialize, Debug)]
pub struct TransferRequest {
    pub id: String,
    pub bank: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: String,
    pub bank: String,
    pub to: String,
    pub transfer_date: String,
    pub amount: u64,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Data {
    pub data: [Transaction; 1],
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ResponseMatch {
    #[serde(rename = "type")]
    pub type_resp: String,
    #[serde(rename = "value")]
    pub value_resp: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Parameters {
    pub body: String,
    pub method: String,
    #[serde(rename = "responseMatches")]
    pub response_matches: [ResponseMatch; 1],
    #[serde(rename = "responseRedactions")]
    pub response_redactions: [ResponseMatch; 0],
    pub url: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Claim {
    pub epoch: u32,
    pub identifier: String,
    pub owner: String,
    #[serde(rename = "timestampS")]
    pub timestamp: u32,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ClaimInfo {
    pub provider: String,
    pub parameters: String,
    pub context: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct SignedClaim {
    pub claim: Claim,
    pub signatures: [String; 1],
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Proof {
    #[serde(rename = "claimInfo")]
    pub claim_info: ClaimInfo,
    #[serde(rename = "signedClaim")]
    pub signed_claim: SignedClaim,
}
