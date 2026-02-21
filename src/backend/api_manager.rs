use crate::backend::provider_config::{get_predefined_providers, get_provider_by_id};
use crate::shared::models::{VgaError, VaultOp, VaultResult, VaultUsageEntry};
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use argon2::Argon2;
use rand::RngCore;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ApiKeyManager {
    vault_path: PathBuf,
    derived_key: Arc<Mutex<Option<[u8; 32]>>>,
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

        Self {
            vault_path,
            derived_key: Arc::new(Mutex::new(None)),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn vault_is_initialized(&self) -> bool {
        let dir = self.vault_dir();
        dir.join("salt.bin").exists() && dir.join("vault_check.enc").exists()
    }

    #[allow(dead_code)]
    pub fn vault_is_unlocked(&self) -> bool {
        self.derived_key
            .lock()
            .map(|g| g.is_some())
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn vault_lock(&self) {
        if let Ok(mut guard) = self.derived_key.lock() {
            *guard = None;
        }
    }

    #[allow(dead_code)]
    pub fn vault_initialize(&self, password: &str) -> Result<(), VgaError> {
        if password.trim().is_empty() {
            return Err(VgaError::AuthVaultError("Password cannot be empty".to_string()));
        }
        if self.vault_is_initialized() {
            return Err(VgaError::AuthVaultError(
                "Vault already initialized".to_string(),
            ));
        }

        let dir = self.vault_dir();
        fs::create_dir_all(dir)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to create vault dir: {e}")))?;

        let mut salt = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        fs::write(dir.join("salt.bin"), &salt)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to write salt: {e}")))?;

        let key = Self::derive_key_from_password(password, &salt)?;
        let check_plain = b"vas-vault-ok";
        let check_encrypted = Self::encrypt_with_key(&key, check_plain)?;
        fs::write(dir.join("vault_check.enc"), check_encrypted)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to write vault check: {e}")))?;

        if let Ok(mut guard) = self.derived_key.lock() {
            *guard = Some(key);
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn vault_unlock(&self, password: &str) -> Result<(), VgaError> {
        if password.trim().is_empty() {
            return Err(VgaError::AuthVaultError("Password cannot be empty".to_string()));
        }
        if !self.vault_is_initialized() {
            return Err(VgaError::AuthVaultError(
                "Vault is not initialized".to_string(),
            ));
        }

        let dir = self.vault_dir();
        let salt = fs::read(dir.join("salt.bin"))
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to read salt: {e}")))?;
        if salt.len() != 16 {
            return Err(VgaError::AuthVaultError("Invalid salt".to_string()));
        }

        let key = Self::derive_key_from_password(password, &salt)?;
        let check_encrypted = fs::read(dir.join("vault_check.enc"))
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to read vault check: {e}")))?;
        let check_plain = Self::decrypt_with_key(&key, &check_encrypted)?;
        if check_plain != b"vas-vault-ok" {
            return Err(VgaError::AuthVaultError("Invalid password".to_string()));
        }

        if let Ok(mut guard) = self.derived_key.lock() {
            *guard = Some(key);
        }
        Ok(())
    }

    pub fn vault_operation(&self, op: VaultOp) -> Result<VaultResult, VgaError> {
        match op {
            VaultOp::Store { provider, key } => {
                self.require_unlocked()?;
                let encrypted = self.encrypt_key(&key)?;
                self.persist_to_disk(&provider, &encrypted)?;
                Ok(VaultResult::Success)
            }
            VaultOp::Retrieve { provider } => {
                self.require_unlocked()?;
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
            VaultOp::GetProviders => {
                let providers = get_predefined_providers();
                Ok(VaultResult::ProviderConfigs(providers))
            }
            VaultOp::GetProviderConfig { provider } => {
                if let Some(config) = get_provider_by_id(&provider) {
                    Ok(VaultResult::ProviderConfig(config))
                } else {
                    Err(VgaError::AuthVaultError(format!("Provider not found: {}", provider)))
                }
            }
            VaultOp::SetDefaultProvider { provider } => {
                self.set_default_provider(&provider)?;
                Ok(VaultResult::DefaultProvider(provider))
            }
        }
    }

    fn set_default_provider(&self, provider: &str) -> Result<(), VgaError> {
        let dir = self.vault_path.parent().unwrap();
        let file_path = dir.join("default_provider.txt");
        fs::write(&file_path, provider.as_bytes())
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to write default provider: {e}")))
    }

    #[allow(dead_code)]
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
    }

    fn encrypt_key(&self, key: &str) -> Result<Vec<u8>, VgaError> {
        let unlocked = self.get_unlocked_key()?;
        Self::encrypt_with_key(&unlocked, key.as_bytes())
    }

    fn decrypt_from_disk(&self, provider: &str) -> Result<String, VgaError> {
        let file_path = self.vault_path.with_file_name(format!("{}.enc", provider));
        let encrypted = fs::read(&file_path)
            .map_err(|e| VgaError::AuthVaultError(format!("Failed to read key file: {}", e)))?;

        let unlocked = self.get_unlocked_key()?;
        let decrypted = Self::decrypt_with_key(&unlocked, &encrypted)?;

        String::from_utf8(decrypted)
            .map_err(|e| VgaError::AuthVaultError(format!("Invalid UTF-8: {}", e)))
    }

    #[allow(dead_code)]
    fn vault_dir(&self) -> &std::path::Path {
        self.vault_path.parent().unwrap()
    }

    fn require_unlocked(&self) -> Result<(), VgaError> {
        if self.vault_is_unlocked() {
            Ok(())
        } else {
            Err(VgaError::AuthVaultError(
                "Vault is locked. Unlock with password to view API keys.".to_string(),
            ))
        }
    }

    fn get_unlocked_key(&self) -> Result<[u8; 32], VgaError> {
        let guard = self
            .derived_key
            .lock()
            .map_err(|_| VgaError::AuthVaultError("Vault key mutex poisoned".to_string()))?;
        guard
            .ok_or_else(|| VgaError::AuthVaultError("Vault is locked".to_string()))
    }

    #[allow(dead_code)]
    fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32], VgaError> {
        let mut out = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), salt, &mut out)
            .map_err(|e| VgaError::AuthVaultError(format!("KDF failed: {e}")))?;
        Ok(out)
    }

    fn encrypt_with_key(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, VgaError> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| VgaError::AuthVaultError(format!("Invalid key: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| VgaError::AuthVaultError(e.to_string()))?;

        let mut out = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn decrypt_with_key(key: &[u8; 32], payload: &[u8]) -> Result<Vec<u8>, VgaError> {
        if payload.len() < 12 {
            return Err(VgaError::AuthVaultError("Encrypted payload too short".to_string()));
        }
        let (nonce_bytes, ciphertext) = payload.split_at(12);

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| VgaError::AuthVaultError(format!("Invalid key: {e}")))?;
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| VgaError::AuthVaultError(format!("Decryption failed: {e}")))
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