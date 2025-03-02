use crate::error::Result;

/// Трейт для криптографического ключа
pub trait Key {
    /// Получить байтовое представление публичного ключа
    fn public_bytes(&self) -> Vec<u8>;
    
    /// Получить байтовое представление приватного ключа (если доступен)
    fn private_bytes(&self) -> Option<Vec<u8>>;
}

/// Трейт для подписи данных
pub trait Signer {
    /// Подписать данные
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
    
    /// Проверить подпись данных
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool>;
}

/// Трейт для шифрования данных
pub trait Cipher {
    /// Зашифровать данные
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    
    /// Расшифровать данные
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}

/// Создать новую пару ключей Ed25519
pub fn generate_ed25519_keypair() -> Result<Box<dyn Key + Send + Sync>> {
    // В реальной реализации здесь будет генерация ключей
    // Для простоты возвращаем заглушку
    Ok(Box::new(ed25519::Ed25519KeyPair::generate()?))
}

/// Создать новую пару ключей X25519 для обмена ключами по Диффи-Хеллману
pub fn generate_x25519_keypair() -> Result<Box<dyn Key + Send + Sync>> {
    // В реальной реализации здесь будет генерация ключей
    // Для простоты возвращаем заглушку
    unimplemented!("Генерация X25519 ключей пока не реализована")
}

/// Хешировать данные с использованием SHA-256
pub fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Хешировать данные с использованием BLAKE3
pub fn blake3(data: &[u8]) -> Vec<u8> {
    blake3::hash(data).as_bytes().to_vec()
}

pub mod ed25519;
pub mod x25519; 