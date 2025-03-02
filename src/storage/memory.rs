use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use super::Storage;

/// Реализация хранилища в памяти
pub struct MemoryStorage {
    /// Имя хранилища
    name: String,
    /// Данные хранилища
    data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MemoryStorage {
    /// Создать новое хранилище в памяти
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut data = self.data.lock()
            .map_err(|_| Error::Storage("Не удалось получить блокировку хранилища".to_string()))?;
        
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }
    
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let data = self.data.lock()
            .map_err(|_| Error::Storage("Не удалось получить блокировку хранилища".to_string()))?;
        
        Ok(data.get(key).cloned())
    }
    
    async fn delete(&mut self, key: &[u8]) -> Result<()> {
        let mut data = self.data.lock()
            .map_err(|_| Error::Storage("Не удалось получить блокировку хранилища".to_string()))?;
        
        data.remove(key);
        Ok(())
    }
    
    async fn has(&self, key: &[u8]) -> Result<bool> {
        let data = self.data.lock()
            .map_err(|_| Error::Storage("Не удалось получить блокировку хранилища".to_string()))?;
        
        Ok(data.contains_key(key))
    }
    
    async fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>> {
        let data = self.data.lock()
            .map_err(|_| Error::Storage("Не удалось получить блокировку хранилища".to_string()))?;
        
        let mut keys = Vec::new();
        for key in data.keys() {
            if key.starts_with(prefix) {
                keys.push(key.clone());
            }
        }
        
        Ok(keys)
    }
    
    async fn close(&mut self) -> Result<()> {
        // Для хранилища в памяти не требуется никаких действий
        Ok(())
    }
}

impl Clone for MemoryStorage {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            data: Arc::clone(&self.data),
        }
    }
} 