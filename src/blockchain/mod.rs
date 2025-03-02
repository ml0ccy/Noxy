use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

use crate::error::Result;
use crate::crypto::Signer;

/// Трейт для блока в блокчейне
pub trait Block: Serialize + for<'de> Deserialize<'de> + Clone + Debug + Send + Sync {
    /// Получить хеш блока
    fn hash(&self) -> Vec<u8>;
    
    /// Получить хеш предыдущего блока
    fn previous_hash(&self) -> &[u8];
    
    /// Получить высоту блока
    fn height(&self) -> u64;
    
    /// Получить метку времени блока
    fn timestamp(&self) -> u64;
    
    /// Проверить валидность блока
    fn is_valid(&self) -> bool;
}

/// Трейт для транзакции в блокчейне
pub trait Transaction: Serialize + for<'de> Deserialize<'de> + Clone + Debug + Send + Sync {
    /// Получить идентификатор транзакции
    fn id(&self) -> Vec<u8>;
    
    /// Подписать транзакцию
    fn sign(&mut self, signer: &dyn Signer) -> Result<()>;
    
    /// Проверить подпись транзакции
    fn verify_signature(&self) -> Result<bool>;
    
    /// Проверить валидность транзакции
    fn is_valid(&self) -> bool;
}

/// Трейт для блокчейна
#[async_trait]
pub trait Blockchain: Send + Sync {
    /// Тип блока
    type BlockType: Block;
    /// Тип транзакции
    type TransactionType: Transaction;
    
    /// Получить последний блок
    async fn get_last_block(&self) -> Result<Self::BlockType>;
    
    /// Получить блок по хешу
    async fn get_block_by_hash(&self, hash: &[u8]) -> Result<Option<Self::BlockType>>;
    
    /// Получить блок по высоте
    async fn get_block_by_height(&self, height: u64) -> Result<Option<Self::BlockType>>;
    
    /// Добавить новый блок
    async fn add_block(&mut self, block: Self::BlockType) -> Result<()>;
    
    /// Добавить новую транзакцию в пул
    async fn add_transaction(&mut self, tx: Self::TransactionType) -> Result<()>;
    
    /// Получить транзакцию по ID
    async fn get_transaction(&self, id: &[u8]) -> Result<Option<Self::TransactionType>>;
    
    /// Получить все транзакции в пуле
    async fn get_transaction_pool(&self) -> Result<Vec<Self::TransactionType>>;
    
    /// Проверить валидность цепочки
    async fn is_chain_valid(&self) -> Result<bool>;
}

pub mod basic; 