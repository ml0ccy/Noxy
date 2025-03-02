use async_trait::async_trait;
use crate::error::Result;

/// Трейт для хранилища данных
#[async_trait]
pub trait Storage: Send + Sync {
    /// Получить имя хранилища
    fn name(&self) -> &str;
    
    /// Сохранить значение по ключу
    async fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
    
    /// Получить значение по ключу
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// Удалить значение по ключу
    async fn delete(&mut self, key: &[u8]) -> Result<()>;
    
    /// Проверить наличие ключа
    async fn has(&self, key: &[u8]) -> Result<bool>;
    
    /// Получить все ключи с определенным префиксом
    async fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>>;
    
    /// Закрыть хранилище
    async fn close(&mut self) -> Result<()>;
}

pub mod memory; 