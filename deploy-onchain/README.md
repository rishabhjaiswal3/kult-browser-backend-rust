# Kult Moments Onchain Deployment

This folder deploys `contracts/KultMomentsActivityRecorder.sol` without setting up a full Hardhat or Foundry project.

## Setup

```sh
cd deploy-onchain
npm install
cp .env.example .env
```

Fill `.env`:

```env
RPC_URL=https://evmrpc.0g.ai/
EXPECTED_CHAIN_ID=16661
DEPLOYER_PRIVATE_KEY=your-admin-wallet-private-key
INITIAL_OWNER=0xYourAdminWalletAddress
RELAYER_ADDRESS=0xBackendRelayerWalletAddress
```

`INITIAL_OWNER` can be left empty; the deployer address will be used.

`RELAYER_ADDRESS` can be left empty; if set, the script grants it permission to call `recordActivity`.

## Deploy

```sh
npm run deploy
```

The script writes:

```txt
deploy-onchain/deployment.json
```

Use the deployed contract address in the backend:

```env
ONCHAIN_ENABLED=true
ONCHAIN_ACTIVITY_CONTRACT=0x...
ONCHAIN_RELAYER_PRIVATE_KEY=your-relayer-private-key
ONCHAIN_CHAIN_ID=16661
ONCHAIN_RPC_URL=https://evmrpc.0g.ai/
```
