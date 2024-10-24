use alloy_sol_types::sol;

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
