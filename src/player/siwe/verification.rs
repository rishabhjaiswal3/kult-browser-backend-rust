use alloy::primitives::Signature;
use std::str::FromStr;

/// Recover the Ethereum address that produced `signature` over `message`.
/// Applies the EIP-191 personal_sign prefix (same as MetaMask `personal_sign`).
pub fn recover_signer(message: &str, signature_hex: &str) -> Result<String, String> {
    let sig = Signature::from_str(signature_hex)
        .map_err(|e| format!("Invalid signature format: {e}"))?;

    let recovered = sig
        .recover_address_from_msg(message.as_bytes())
        .map_err(|e| format!("Signature recovery failed: {e}"))?;

    Ok(format!("{recovered}"))
}

/// Verify that `signature` over `message` was produced by `wallet_address`.
pub fn verify_wallet_signature(
    wallet_address: &str,
    message: &str,
    signature_hex: &str,
) -> Result<(), String> {
    let recovered = recover_signer(message, signature_hex)?;

    if recovered.to_lowercase() != wallet_address.to_lowercase() {
        return Err(format!(
            "Signature mismatch: message was signed by {}, not {}",
            recovered.to_lowercase(),
            wallet_address.to_lowercase()
        ));
    }

    Ok(())
}

/// Extract the nonce value from a SIWE message body.
/// Looks for a line starting with "Nonce: ".
pub fn extract_nonce(message: &str) -> Option<&str> {
    message
        .lines()
        .find(|l| l.starts_with("Nonce: "))
        .map(|l| l.trim_start_matches("Nonce: ").trim())
}
