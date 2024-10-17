use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Serialize, Debug)]
pub struct TransferRequest {
    pub id: String,
    pub bank: String,
}

#[derive(Default, Deserialize, Debug)]
pub struct ClaimData {
    pub provider: String,
    pub parameters: String,
    pub owner: String,
    #[serde(rename = "timestampS")] 
    pub timestamp: u64,
    pub context: String,
    pub identifier: String,
    pub epoch: u64,
}

#[derive(Default, Deserialize, Debug)]
pub struct Witness {
    pub id: String,
    pub url: String,
}

#[derive(Default, Deserialize, Debug)]
pub struct TransferResponse {
    #[serde(rename = "claimData")] 
    pub claim_data: ClaimData,
    pub identifier: String,
    pub signatures: [String; 1],
    #[serde(rename = "extractedParameterValues")] 
    pub extracted_parameter_values: String,
    pub witnesses: [Witness; 1],
}

#[derive(Default, Deserialize, Debug)]
pub struct Parameters {
    pub body: String,
    pub method: String,
    #[serde(rename = "responseMatches")] 
    pub response_matches: [ResponseMatch; 1]
}

#[derive(Default, Deserialize, Debug)]
pub struct TransferData {
    pub id: String,
    pub bank: String,
    pub to: String,
    pub transfer_date: String,
    pub amount: u64,
}

#[derive(Default, Deserialize, Debug)]
pub struct Data {
    pub data: [TransferData; 1],
}

#[derive(Default, Deserialize, Debug)]
pub struct ResponseMatch {
    #[serde(rename = "type")] 
    pub type_resp: String,
    #[serde(rename = "value")] 
    pub value_resp: String,
}
