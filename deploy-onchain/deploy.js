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
const OUTPUT_FILE = path.resolve(__dirname, "deployment.json");

function requiredEnv(name) {
  const value = process.env[name]?.trim();
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
}

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
          "*": ["abi", "evm.bytecode.object"],
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
    abi: compiled.abi,
    bytecode: `0x${compiled.evm.bytecode.object}`,
  };
}

async function main() {
  const rpcUrl = requiredEnv("RPC_URL");
  const privateKey = requiredEnv("DEPLOYER_PRIVATE_KEY");
  const expectedChainId = BigInt(process.env.EXPECTED_CHAIN_ID || "16661");

  const provider = new ethers.JsonRpcProvider(rpcUrl);
  const network = await provider.getNetwork();

  if (network.chainId !== expectedChainId) {
    throw new Error(
      `Wrong network. Expected chain ID ${expectedChainId}, got ${network.chainId}`,
    );
  }

  const wallet = new ethers.Wallet(privateKey, provider);
  const deployerAddress = await wallet.getAddress();
  const initialOwner = process.env.INITIAL_OWNER?.trim() || deployerAddress;
  const relayerAddress = process.env.RELAYER_ADDRESS?.trim();

  console.log(`Network chain ID: ${network.chainId}`);
  console.log(`Deployer: ${deployerAddress}`);
  console.log(`Initial owner: ${initialOwner}`);

  const { abi, bytecode } = compileContract();
  const factory = new ethers.ContractFactory(abi, bytecode, wallet);

  console.log(`Deploying ${CONTRACT_NAME}...`);
  const contract = await factory.deploy(initialOwner);
  await contract.waitForDeployment();

  const contractAddress = await contract.getAddress();
  const deployTx = contract.deploymentTransaction();

  console.log(`Contract deployed: ${contractAddress}`);
  console.log(`Deploy tx: ${deployTx.hash}`);

  let writerGrantTxHash = null;

  if (relayerAddress) {
    console.log(`Granting activity writer role to ${relayerAddress}...`);
    const tx = await contract.setActivityWriter(relayerAddress, true);
    await tx.wait();
    writerGrantTxHash = tx.hash;
    console.log(`Writer grant tx: ${writerGrantTxHash}`);
  } else {
    console.log("RELAYER_ADDRESS not set; skipping writer grant");
  }

  const deployment = {
    contractName: CONTRACT_NAME,
    contractAddress,
    deployTxHash: deployTx.hash,
    writerGrantTxHash,
    chainId: network.chainId.toString(),
    rpcUrl,
    deployerAddress,
    initialOwner,
    relayerAddress: relayerAddress || null,
    deployedAt: new Date().toISOString(),
  };

  fs.writeFileSync(OUTPUT_FILE, `${JSON.stringify(deployment, null, 2)}\n`);
  console.log(`Deployment saved: ${OUTPUT_FILE}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
