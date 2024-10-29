const express = require("express");
let { ReclaimClient } = require("@reclaimprotocol/zk-fetch");
let { Reclaim } = require("@reclaimprotocol/js-sdk");
const dotenv = require("dotenv");
let mutation_result = require("./mutation.json");

dotenv.config();

const reclaimClient = new ReclaimClient(
  process.env.APP_ID,
  process.env.APP_SECRET,
  true
);

const app = express();
app.use(express.json());
const port = 7787;

async function generateProof(url) {
  try {
    const proof = await reclaimClient.zkFetch(
      url,
      {
        method: "GET",
      },
      3,
      5000
    );

    if (!proof) {
      return {
        success: false,
        error: "Failed to generate proof",
      };
    }

    const isValid = await Reclaim.verifySignedProof(proof);
    if (!isValid) {
      return {
        success: false,
        error: "Proof is invalid",
      };
    }

    const proofData = await Reclaim.transformForOnchain(proof);

    return {
      success: true,
      data: proofData,
    };
  } catch (err) {
    console.log(err);

    return {
      success: false,
      error: err.message,
    };
  }
}

app.post("/generateTransferProof", async (req, res) => {
  if (!req.headers.authorization) {
    return res.status(403).json({ error: "No credentials sent!" });
  }

  try {
    const result = await generateProof(
      `https://mock.blocknaut.xyz/api/v2/mutation?bank=${req.body.bank}&id=${req.body.id}`
    );

    if (!result.success) {
      console.log(result);
      return res.status(400).send(result.error);
    }

    return res.status(200).json(result.data);
  } catch (e) {
    console.log(e);
    return res.status(500).send(e);
  }
});

app.get("/api/v2/mutation", (req, res) => {
  if (req.query.result == "error") {
    return res.status(401).send({ error: "No Authorization" });
  } else if (req.query.result == "notfound") {
    return res.send({ data: [] });
  } else if (req.query.id != undefined && req.query.bank != undefined) {
    let item = {};

    function search(value, index, array) {
      if (
        value.id == req.query.id &&
        value.bank == req.query.bank.toLowerCase()
      ) {
        item = value;
      }
    }

    mutation_result.data.forEach(search);
    return res.send({ data: [item] });
  } else if (req.query.result == "all") {
    return res.send(mutation_result);
  } else {
    return res.status(404).send({ error: "'bank' and 'id' is mandatory" });
  }
});

app.listen(port, () => {
  console.log(`Mock API listening on port ${port}`);
});
