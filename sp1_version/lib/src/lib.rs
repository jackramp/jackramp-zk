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
    struct ReclaimProof {
        bytes32 hashedClaimInfo;
        SignedClaim signedClaim;
    }
  }
  
  sol! {
    #[derive(Debug)]
    struct OfframpRequestParams {
        address user;
        uint256 amount;
        uint256 amountRealWorld;
        bytes32 hashedChannelAccount;
        bytes32 hashedChannelId;
    }
  }
  
  sol! {
    #[derive(Debug)]
    struct PublicValuesStruct {
        OfframpRequestParams offrampRequestParams;
        ReclaimProof proof;
    }
  }