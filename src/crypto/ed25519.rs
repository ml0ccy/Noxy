use ed25519_dalek::{Signer as Ed25519Signer, Verifier, Signature, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use crate::error::{Error, Result};
use super::{Key, Signer};

/// Пара ключей Ed25519
pub struct Ed25519KeyPair {
    /// Приватный ключ для подписи
    private_key: Option<SigningKey>,
    /// Публичный ключ для проверки
    public_key: VerifyingKey,
}

impl Ed25519KeyPair {
    /// Создать новую пару ключей
    pub fn generate() -> Result<Self> {
        let private_key = SigningKey::generate(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);
        
        Ok(Self {
            private_key: Some(private_key),
            public_key,
        })
    }
    
    /// Создать пару ключей из существующего приватного ключа
    pub fn from_private_key(private_bytes: &[u8]) -> Result<Self> {
        if private_bytes.len() != 32 {
            return Err(Error::Crypto("Некорректная длина приватного ключа Ed25519".to_string()));
        }
        
        let private_key = SigningKey::from_bytes(
            &private_bytes.try_into().map_err(|_| {
                Error::Crypto("Не удалось преобразовать байты в ключ Ed25519".to_string())
            })?
        );
        
        let public_key = VerifyingKey::from(&private_key);
        
        Ok(Self {
            private_key: Some(private_key),
            public_key,
        })
    }
    
    /// Создать пару ключей только с публичным ключом (для проверки)
    pub fn from_public_key(public_bytes: &[u8]) -> Result<Self> {
        if public_bytes.len() != 32 {
            return Err(Error::Crypto("Некорректная длина публичного ключа Ed25519".to_string()));
        }
        
        let public_key = VerifyingKey::from_bytes(
            &public_bytes.try_into().map_err(|_| {
                Error::Crypto("Не удалось преобразовать байты в публичный ключ Ed25519".to_string())
            })?
        ).map_err(|e| Error::Crypto(format!("Ошибка при создании публичного ключа: {}", e)))?;
        
        Ok(Self {
            private_key: None,
            public_key,
        })
    }
}

impl Key for Ed25519KeyPair {
    fn public_bytes(&self) -> Vec<u8> {
        self.public_key.to_bytes().to_vec()
    }
    
    fn private_bytes(&self) -> Option<Vec<u8>> {
        self.private_key.as_ref().map(|pk| pk.to_bytes().to_vec())
    }
}

impl Signer for Ed25519KeyPair {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(private_key) = &self.private_key {
            let signature = private_key.sign(data);
            Ok(signature.to_bytes().to_vec())
        } else {
            Err(Error::Crypto("Отсутствует приватный ключ для подписи".to_string()))
        }
    }
    
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        if signature.len() != 64 {
            return Err(Error::Crypto("Некорректная длина подписи Ed25519".to_string()));
        }
        
        let sig = Signature::from_bytes(
            &signature.try_into().map_err(|_| {
                Error::Crypto("Не удалось преобразовать байты в подпись Ed25519".to_string())
            })?
        );
        
        match self.public_key.verify(data, &sig) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
} 