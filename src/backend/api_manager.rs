use crate::shared::models::{VgaError, VaultOp, VaultResult};
use aes_gcm::{Aes256Gcm, Nonce, aead::Aead, KeyInit};
use rand::RngCore;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::shared::models::VaultUsageEntry;

#[derive(Clone)]
pub struct ApiKeyManager {
    vault_path: PathBuf,
    master_key: Vec<u8>,
    usage_stats: Arc<RwLock<HashMap<String, UsageStats>>>,
}

#[derive(Clone, Debug)]
struct UsageStats {
    tokens_used: u64,
    requests_made: u64,
    last_used: chrono::DateTime<chrono::Utc>,
}

impl ApiKeyManager {
    pub async fn new() -> Self {
        let vault_path = PathBuf::from("vault/keys.enc");
        fs::create_dir_all(&vault_path.parent().unwrap()).unwrap();

        // In production, this should be securely generated and stored
        let master_key = b"an example very very secret key."; // 32 bytes

        Self {
            vault_path,
            master_key: master_key.to_vec(),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn vault_operation(&self, op: VaultOp) -> Result<VaultResult, VgaError> {
        match op {
            VaultOp::Store { provider, key } => {
                let encrypted = self.encrypt_key(&key)?;
                self.persist_to_disk(&provider, &encrypted)?;
                Ok(VaultResult::Success)
            }
            VaultOp::Retrieve { provider } => {
                let decrypted = self.decrypt_from_disk(&provider)?;
                self.update_usage_stats(&provider);
                Ok(VaultResult::Key(decrypted))
            }
            VaultOp::Delete { provider } => {
                self.delete_from_disk(&provider)?;
                Ok(VaultResult::Success)
            }
            VaultOp::List => {
                let keys = self.list_providers()?;
                Ok(VaultResult::Providers(keys))
            }
        }
    }

    pub fn get_decrypted_key(&self, provider: &str) -> Result<String, VgaError> {
        self.vault_operation(VaultOp::Retrieve { provider: provider.to_string() })
            .and_then(|res| match res {
                VaultResult::Key(key) => Ok(key),
                _ => Err(VgaError::AuthVaultError("Unexpected result".into())),
            })
    }

    pub fn update_usage_stats(&self, provider: &str) {
        let mut stats = self.usage_stats.blocking_write();
        let entry = stats.entry(provider.to_string()).or_insert(UsageStats {
            tokens_used: 0,
            requests_made: 0,
            last_used: chrono::Utc::now(),
        });
        entry.requests_made += 1;
        entry.last_used = chrono::Utc::now();
    }

    pub async fn get_usage_entries(&self) -> Vec<VaultUsageEntry> {
        let stats = self.usage_stats.read().await;
        let mut out: Vec<VaultUsageEntry> = stats
            .iter()
            .map(|(provider, s)| VaultUsageEntry {
                provider: provider.clone(),
                requests_made: s.requests_made,
                last_used: s.last_used,
            })
            .collect();
        for entry in stats.values() {
            let _ = entry.tokens_used;
        }
        out.sort_by(|a, b| a.provider.cmp(&b.provider));
        out
    }

    pub fn check_quota_availability(&self, _provider: &str) -> bool {
        // TODO: Implement actual quota checking
        true
    }

    pub fn prime_demo_usage(&self) {
        let _ = self.check_quota_availability("local");
        let _ = self.get_decrypted_key("local");
    }

    fn encrypt_key(&self, key: &str) -> Result<Vec<u8>, VgaError> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| VgaError::AuthVaultError(format!("Invalid key: {}", e)))?;
        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, key.as_bytes())
            .map_err(|e| VgaError::AuthVaultError(e.to_string()))
            ?;

        // Store nonce + ciphertext so we can decrypt later.
        let mut out = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn decrypt_from_disk(&self, provider: &str) -> Result<String, VgaError> {
        let file_path = self.vault_path.with_file_name(format!("{}.enc", provider));
        let encrypted = fs::read(&file_path)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to read key file: {}", e)))?;

        if encrypted.len() < 12 {
            return Err(VgaError::AuthVaultError("Encrypted payload too short".to_string()));
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(12);

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| VgaError::AuthVaultError(format!("Invalid key: {}", e)))?;
        let nonce = Nonce::from_slice(nonce_bytes);
        let decrypted = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| VgaError::AuthVaultError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(decrypted)
            .map_err(|e| VgaError::AuthVaultError(format!("Invalid UTF-8: {}", e)))
    }

    fn persist_to_disk(&self, provider: &str, encrypted: &[u8]) -> Result<(), VgaError> {
        let file_path = self.vault_path.with_file_name(format!("{}.enc", provider));
        fs::write(&file_path, encrypted)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to write key file: {}", e)))
    }

    fn delete_from_disk(&self, provider: &str) -> Result<(), VgaError> {
        let file_path = self.vault_path.with_file_name(format!("{}.enc", provider));
        fs::remove_file(&file_path)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to delete key file: {}", e)))
    }

    fn list_providers(&self) -> Result<Vec<String>, VgaError> {
        let dir = self.vault_path.parent().unwrap();
        if !dir.exists() {
            return Ok(vec![]);
        }
        let entries = fs::read_dir(dir)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to read vault directory: {}", e)))?;
        let mut providers = vec![];
        for entry in entries {
            let entry = entry.map_err(|e| VgaError::AuthVaultError(e.to_string()))?;
            if let Some(ext) = entry.path().extension() {
                if ext == "enc" {
                    if let Some(stem) = entry.path().file_stem() {
                        providers.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(providers)
    }
}