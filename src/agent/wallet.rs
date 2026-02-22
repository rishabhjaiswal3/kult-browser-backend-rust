use alloy::signers::local::PrivateKeySigner;

/// Represents an AI agent's automatically generated EVM wallet.
#[derive(Debug, Clone)]
pub struct AgentWallet {
    pub address: String,
    pub private_key: String,
}

impl AgentWallet {
    /// Generates a completely new, secure random Ethereum wallet for an AI agent.
    /// This should be called during user registration to pair an agent wallet with the user.
    pub fn generate() -> Self {
        // Generate a random secp256k1 private key securely
        let signer = PrivateKeySigner::random();

        // alloy's signer natively exposes the public address
        let address = signer.address().to_string();

        // Extract the raw hex private key (without the '0x' prefix for standard storage/use)
        let private_key = alloy::hex::encode(signer.to_bytes());

        AgentWallet {
            address,
            private_key,
        }
    }

    /// Reconstructs an AgentWallet instance from a previously stored hex private key.
    pub fn from_private_key(hex_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let signer: PrivateKeySigner = hex_key.parse()?;
        let address = signer.address().to_string();

        Ok(AgentWallet {
            address,
            private_key: hex_key.to_string(),
        })
    }
}
