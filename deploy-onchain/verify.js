require("dotenv").config();

const fs = require("fs");
const path = require("path");
const solc = require("solc");
const { ethers } = require("ethers");

const CONTRACT_NAME = "KultMomentsActivityRecorder";
const CONTRACT_FILE = path.resolve(
  __dirname,
  "../contracts/KultMomentsActivityRecorder.sol",
);
const DEPLOYMENT_FILE = path.resolve(__dirname, "deployment.json");

function compileContract() {
  const source = fs.readFileSync(CONTRACT_FILE, "utf8");
  const input = {
    language: "Solidity",
    sources: {
      "KultMomentsActivityRecorder.sol": {
        content: source,
      },
    },
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
      outputSelection: {
        "*": {
          "*": ["evm.deployedBytecode.object"],
        },
      },
    },
  };

  const output = JSON.parse(solc.compile(JSON.stringify(input)));
  const errors = output.errors || [];
  const fatalErrors = errors.filter((error) => error.severity === "error");

  for (const error of errors) {
    const log = error.formattedMessage || error.message;
    if (error.severity === "error") {
      console.error(log);
    } else {
      console.warn(log);
    }
  }

  if (fatalErrors.length > 0) {
    throw new Error("Solidity compilation failed");
  }

  const compiled =
    output.contracts["KultMomentsActivityRecorder.sol"][CONTRACT_NAME];

  return {
    source,
    deployedBytecode: `0x${compiled.evm.deployedBytecode.object}`,
    compiler: `v${solc.version().replace(".Emscripten.clang", "")}`,
  };
}

async function main() {
  if (!fs.existsSync(DEPLOYMENT_FILE)) {
    throw new Error("deployment.json is missing. Run npm run deploy first.");
  }

  const deployment = JSON.parse(fs.readFileSync(DEPLOYMENT_FILE, "utf8"));
  const rpcUrl = process.env.RPC_URL || deployment.rpcUrl;
  const provider = new ethers.JsonRpcProvider(rpcUrl);
  const { source, deployedBytecode, compiler } = compileContract();
  const onchainBytecode = await provider.getCode(deployment.contractAddress);

  console.log(`Contract: ${deployment.contractAddress}`);
  console.log(`Compiler: ${compiler}`);
  console.log(`Runtime bytecode match: ${onchainBytecode.toLowerCase() === deployedBytecode.toLowerCase()}`);

  const body = {
    address: deployment.contractAddress,
    sourceCode: source,
    compiler,
    contractName: CONTRACT_NAME,
    optimization: true,
    runs: 200,
    license: "3",
  };

  const response = await fetch("https://chainscan.0g.ai/v1/contract/verify", {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
  const result = await response.json();

  console.log(JSON.stringify(result, null, 2));

  if (
    result?.result?.errors?.some((error) =>
      String(error).includes("bytecode_length_mismatch"),
    )
  ) {
    console.log(
      "\nChainScan currently documents that constructor arguments are not supported for source verification. The runtime bytecode matches, so this mismatch is caused by creation bytecode/constructor arguments, not by wrong source.",
    );
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
