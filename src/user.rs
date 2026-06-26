//! Local user accounts (login) with salted SHA-256 password hashing.
//!
//! These are local profiles, not networked auth — they scope per-user practice
//! progress and a "currently signed-in" user for the CLI and TUI.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// One registered account.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub username: String,
    pub salt: String,
    pub password_hash: String,
    #[serde(default)]
    pub created: String,
}

/// Hash a password with its per-user salt.
pub fn hash_password(salt: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b"$maestro$");
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// The collection of accounts plus the currently signed-in user.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStore {
    #[serde(default)]
    pub users: BTreeMap<String, User>,
    #[serde(default)]
    pub current: Option<String>,
}

impl UserStore {
    pub fn path() -> PathBuf {
        crate::config::state_dir().join("users.json")
    }

    pub fn load() -> Result<UserStore> {
        let path = Self::path();
        if !path.exists() {
            return Ok(UserStore::default());
        }
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Register a new account. Fails if the username is taken or empty.
    pub fn register(
        &mut self,
        username: &str,
        password: &str,
        salt: &str,
        created: &str,
    ) -> Result<()> {
        let username = username.trim();
        if username.is_empty() {
            bail!("username must not be empty");
        }
        if password.is_empty() {
            bail!("password must not be empty");
        }
        if self.users.contains_key(username) {
            bail!("user '{}' already exists", username);
        }
        self.users.insert(
            username.to_string(),
            User {
                username: username.to_string(),
                salt: salt.to_string(),
                password_hash: hash_password(salt, password),
                created: created.to_string(),
            },
        );
        Ok(())
    }

    /// Verify a username/password pair.
    pub fn verify(&self, username: &str, password: &str) -> bool {
        match self.users.get(username.trim()) {
            Some(u) => u.password_hash == hash_password(&u.salt, password),
            None => false,
        }
    }

    /// Sign in, recording the current user. Fails on bad credentials.
    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        if !self.verify(username, password) {
            bail!("invalid username or password");
        }
        self.current = Some(username.trim().to_string());
        Ok(())
    }

    /// Sign out the current user.
    pub fn logout(&mut self) {
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_then_verify() {
        let mut store = UserStore::default();
        store
            .register("hector", "piano123", "saltA", "2025-05-11")
            .unwrap();
        assert!(store.verify("hector", "piano123"));
        assert!(!store.verify("hector", "wrong"));
        assert!(!store.verify("ghost", "piano123"));
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut store = UserStore::default();
        store.register("a", "p", "s", "d").unwrap();
        assert!(store.register("a", "q", "s", "d").is_err());
    }

    #[test]
    fn empty_credentials_rejected() {
        let mut store = UserStore::default();
        assert!(store.register("", "p", "s", "d").is_err());
        assert!(store.register("a", "", "s", "d").is_err());
    }

    #[test]
    fn login_logout_flow() {
        let mut store = UserStore::default();
        store
            .register("hector", "piano123", "saltA", "2025-05-11")
            .unwrap();
        assert!(store.login("hector", "piano123").is_ok());
        assert_eq!(store.current.as_deref(), Some("hector"));
        store.logout();
        assert!(store.current.is_none());
        assert!(store.login("hector", "nope").is_err());
    }

    #[test]
    fn hash_is_salt_sensitive() {
        assert_ne!(hash_password("s1", "pw"), hash_password("s2", "pw"));
        assert_eq!(hash_password("s1", "pw"), hash_password("s1", "pw"));
    }
}
