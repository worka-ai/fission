use anyhow::{bail, Context, Result};
use base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD};
use fission_core::{Action, ActionEnvelope};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedServerAction {
    pub route_path: String,
    pub target_node: u128,
    pub action: ActionEnvelope,
    pub expires_unix: u64,
    pub nonce: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedServerAction {
    pub route_path: String,
    pub target_node: u128,
    pub action: ActionEnvelope,
}

#[derive(Clone)]
pub struct ServerActionSigner {
    key: [u8; 32],
}

impl ServerActionSigner {
    pub fn new(secret: impl AsRef<[u8]>) -> Self {
        let hash = blake3::hash(secret.as_ref());
        Self {
            key: *hash.as_bytes(),
        }
    }

    pub fn development() -> Self {
        Self::new(b"fission-development-server-action-key")
    }

    pub fn sign<A: Action>(
        &self,
        route_path: impl Into<String>,
        target_node: u128,
        action: A,
        ttl: Duration,
    ) -> SignedServerAction {
        self.sign_envelope(route_path, target_node, action.into(), ttl)
    }

    pub fn sign_envelope(
        &self,
        route_path: impl Into<String>,
        target_node: u128,
        action: ActionEnvelope,
        ttl: Duration,
    ) -> SignedServerAction {
        let route_path = route_path.into();
        let expires_unix = unix_now().saturating_add(ttl.as_secs());
        let nonce = nonce_for(&route_path, target_node, &action, expires_unix);
        let signature = self.signature(&route_path, target_node, &action, expires_unix, &nonce);
        SignedServerAction {
            route_path,
            target_node,
            action,
            expires_unix,
            nonce,
            signature,
        }
    }

    pub fn verify(&self, token: &SignedServerAction) -> Result<VerifiedServerAction> {
        if token.expires_unix < unix_now() {
            bail!("server action token expired");
        }
        let expected = self.signature(
            &token.route_path,
            token.target_node,
            &token.action,
            token.expires_unix,
            &token.nonce,
        );
        if !constant_time_eq(expected.as_bytes(), token.signature.as_bytes()) {
            bail!("server action token signature mismatch");
        }
        Ok(VerifiedServerAction {
            route_path: token.route_path.clone(),
            target_node: token.target_node,
            action: token.action.clone(),
        })
    }

    pub fn encode(&self, token: &SignedServerAction) -> Result<String> {
        let bytes = serde_json::to_vec(token).context("failed to encode server action token")?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn decode(&self, encoded: &str) -> Result<SignedServerAction> {
        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(encoded)
            .context("failed to decode server action token")?;
        serde_json::from_slice(&bytes).context("failed to parse server action token")
    }

    fn signature(
        &self,
        route_path: &str,
        target_node: u128,
        action: &ActionEnvelope,
        expires_unix: u64,
        nonce: &str,
    ) -> String {
        let mut hasher = blake3::Hasher::new_keyed(&self.key);
        hasher.update(b"fission.server.action.v1");
        hasher.update(route_path.as_bytes());
        hasher.update(&target_node.to_le_bytes());
        hasher.update(&action.id.as_u128().to_le_bytes());
        hasher.update(&(action.payload.len() as u64).to_le_bytes());
        hasher.update(&action.payload);
        hasher.update(&expires_unix.to_le_bytes());
        hasher.update(nonce.as_bytes());
        to_hex(hasher.finalize().as_bytes())
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn nonce_for(
    route_path: &str,
    target_node: u128,
    action: &ActionEnvelope,
    expires_unix: u64,
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"fission.server.action.nonce.v1");
    hasher.update(route_path.as_bytes());
    hasher.update(&target_node.to_le_bytes());
    hasher.update(&action.id.as_u128().to_le_bytes());
    hasher.update(&action.payload);
    hasher.update(&expires_unix.to_le_bytes());
    hasher.update(&now.as_nanos().to_le_bytes());
    hasher.update(&std::process::id().to_le_bytes());
    to_hex(&hasher.finalize().as_bytes()[..16])
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use fission_core::{Action, ActionId};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct AddToCart {
        sku: String,
    }

    impl Action for AddToCart {
        fn static_id() -> ActionId {
            ActionId::from_name("test::AddToCart")
        }
    }

    #[test]
    fn signed_action_tokens_round_trip_and_reject_tampering() {
        let signer = ServerActionSigner::new("secret");
        let token = signer.sign(
            "/",
            7,
            AddToCart { sku: "abc".into() },
            Duration::from_secs(60),
        );
        let encoded = signer.encode(&token).unwrap();
        let decoded = signer.decode(&encoded).unwrap();
        assert_eq!(signer.verify(&decoded).unwrap().target_node, 7);

        let mut tampered = decoded;
        tampered.target_node = 8;
        assert!(signer.verify(&tampered).is_err());
    }
}
