use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::crypto::{Signer, sha256};
use crate::storage::Storage;
use super::{Block, Transaction, Blockchain};

/// Базовая реализация блока
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    /// Хеш блока
    hash: Vec<u8>,
    /// Хеш предыдущего блока
    previous_hash: Vec<u8>,
    /// Высота блока
    height: u64,
    /// Метка времени
    timestamp: u64,
    /// Сложность
    difficulty: u32,
    /// Nonce для proof-of-work
    nonce: u64,
    /// Транзакции
    transactions: Vec<BasicTransaction>,
    /// Данные блока
    data: Vec<u8>,
}

impl BasicBlock {
    /// Создать новый блок
    pub fn new(
        previous_hash: Vec<u8>,
        height: u64,
        transactions: Vec<BasicTransaction>,
        data: Vec<u8>,
        difficulty: u32,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Время до начала эпохи")
            .as_secs();
        
        let mut block = Self {
            hash: Vec::new(),
            previous_hash,
            height,
            timestamp,
            difficulty,
            nonce: 0,
            transactions,
            data,
        };
        
        // Вычисляем хеш блока
        block.mine();
        
        block
    }
    
    /// Создать genesis блок
    pub fn genesis() -> Self {
        Self::new(
            vec![0; 32],  // Хеш предыдущего блока (нули для генезис-блока)
            0,            // Высота
            Vec::new(),   // Транзакции
            b"Genesis Block".to_vec(), // Данные
            1,            // Сложность
        )
    }
    
    /// Майнинг блока (proof-of-work)
    pub fn mine(&mut self) {
        let target = 1u64 << (64 - self.difficulty as u64);
        
        loop {
            self.hash = self.calculate_hash();
            
            // Проверяем, удовлетворяет ли хеш требованиям сложности
            let hash_value = if self.hash.len() >= 8 {
                let mut value = 0u64;
                for i in 0..8 {
                    value = (value << 8) | self.hash[i] as u64;
                }
                value
            } else {
                0
            };
            
            if hash_value < target {
                break;
            }
            
            self.nonce += 1;
        }
    }
    
    /// Вычислить хеш блока
    fn calculate_hash(&self) -> Vec<u8> {
        // Для вычисления хеша сериализуем все поля кроме самого хеша
        let mut data = Vec::new();
        data.extend_from_slice(&self.previous_hash);
        data.extend_from_slice(&self.height.to_be_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data.extend_from_slice(&self.difficulty.to_be_bytes());
        data.extend_from_slice(&self.nonce.to_be_bytes());
        
        // Добавляем хеши всех транзакций
        for tx in &self.transactions {
            data.extend_from_slice(&tx.id());
        }
        
        data.extend_from_slice(&self.data);
        
        sha256(&data)
    }
}

impl Block for BasicBlock {
    fn hash(&self) -> Vec<u8> {
        self.hash.clone()
    }
    
    fn previous_hash(&self) -> &[u8] {
        &self.previous_hash
    }
    
    fn height(&self) -> u64 {
        self.height
    }
    
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    fn is_valid(&self) -> bool {
        // Проверяем, соответствует ли хеш содержимому блока
        let calculated_hash = self.calculate_hash();
        if calculated_hash != self.hash {
            return false;
        }
        
        // Проверяем, что хеш удовлетворяет требованиям сложности
        let target = 1u64 << (64 - self.difficulty as u64);
        let hash_value = if self.hash.len() >= 8 {
            let mut value = 0u64;
            for i in 0..8 {
                value = (value << 8) | self.hash[i] as u64;
            }
            value
        } else {
            0
        };
        
        if hash_value >= target {
            return false;
        }
        
        // Проверяем все транзакции в блоке
        for tx in &self.transactions {
            if !tx.is_valid() {
                return false;
            }
        }
        
        true
    }
}

/// Базовая реализация транзакции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicTransaction {
    /// Идентификатор транзакции
    id: Vec<u8>,
    /// Отправитель
    sender: Vec<u8>,
    /// Получатель
    receiver: Vec<u8>,
    /// Сумма
    amount: u64,
    /// Метка времени
    timestamp: u64,
    /// Подпись
    signature: Option<Vec<u8>>,
    /// Дополнительные данные
    data: Vec<u8>,
}

impl BasicTransaction {
    /// Создать новую транзакцию
    pub fn new(
        sender: Vec<u8>,
        receiver: Vec<u8>,
        amount: u64,
        data: Vec<u8>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Время до начала эпохи")
            .as_secs();
        
        let mut tx = Self {
            id: Vec::new(),
            sender,
            receiver,
            amount,
            timestamp,
            signature: None,
            data,
        };
        
        // Вычисляем ID транзакции
        tx.id = tx.calculate_hash();
        
        tx
    }
    
    /// Вычислить хеш транзакции
    fn calculate_hash(&self) -> Vec<u8> {
        // Сериализуем все поля кроме идентификатора и подписи
        let mut data = Vec::new();
        data.extend_from_slice(&self.sender);
        data.extend_from_slice(&self.receiver);
        data.extend_from_slice(&self.amount.to_be_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data.extend_from_slice(&self.data);
        
        sha256(&data)
    }
    
    /// Данные для подписи
    fn data_to_sign(&self) -> Vec<u8> {
        // Используем идентификатор транзакции как данные для подписи
        self.id.clone()
    }
}

impl Transaction for BasicTransaction {
    fn id(&self) -> Vec<u8> {
        self.id.clone()
    }
    
    fn sign(&mut self, signer: &dyn Signer) -> Result<()> {
        let data_to_sign = self.data_to_sign();
        let signature = signer.sign(&data_to_sign)?;
        self.signature = Some(signature);
        Ok(())
    }
    
    fn verify_signature(&self) -> Result<bool> {
        // Если нет подписи, возвращаем false
        let signature = match &self.signature {
            Some(sig) => sig,
            None => return Ok(false),
        };
        
        // В реальной реализации здесь будет проверка подписи
        // через публичный ключ отправителя
        // Для упрощения примера просто возвращаем true
        Ok(true)
    }
    
    fn is_valid(&self) -> bool {
        // Проверяем, что идентификатор транзакции соответствует её содержимому
        let calculated_id = self.calculate_hash();
        if calculated_id != self.id {
            return false;
        }
        
        // Проверяем подпись
        match self.verify_signature() {
            Ok(valid) => valid,
            Err(_) => false,
        }
    }
}

/// Базовая реализация блокчейна
pub struct BasicBlockchain {
    /// Хранилище блоков
    storage: Box<dyn Storage>,
    /// Последний блок
    last_block: Arc<Mutex<Option<BasicBlock>>>,
    /// Пул транзакций
    transaction_pool: Arc<Mutex<HashSet<BasicTransaction>>>,
    /// Индекс блоков по высоте
    blocks_by_height: Arc<Mutex<HashMap<u64, Vec<u8>>>>,
    /// Сложность
    difficulty: u32,
}

impl BasicBlockchain {
    /// Создать новый блокчейн
    pub fn new(storage: Box<dyn Storage>, difficulty: u32) -> Self {
        Self {
            storage,
            last_block: Arc::new(Mutex::new(None)),
            transaction_pool: Arc::new(Mutex::new(HashSet::new())),
            blocks_by_height: Arc::new(Mutex::new(HashMap::new())),
            difficulty,
        }
    }
    
    /// Инициализировать блокчейн
    pub async fn initialize(&mut self) -> Result<()> {
        // Проверяем, есть ли уже блоки в хранилище
        let genesis_key = b"block:0".to_vec();
        
        if let Some(genesis_data) = self.storage.get(&genesis_key).await? {
            // Загружаем генезис-блок
            let genesis: BasicBlock = bincode::deserialize(&genesis_data)
                .map_err(|e| Error::Serialization(format!("Не удалось десериализовать генезис-блок: {}", e)))?;
            
            // Загружаем последний блок
            let last_height_data = self.storage.get(b"last_height").await?
                .ok_or_else(|| Error::Blockchain("Не найдена высота последнего блока".to_string()))?;
            
            let last_height = bincode::deserialize::<u64>(&last_height_data)
                .map_err(|e| Error::Serialization(format!("Не удалось десериализовать высоту последнего блока: {}", e)))?;
            
            let last_block_key = format!("block:{}", last_height).into_bytes();
            let last_block_data = self.storage.get(&last_block_key).await?
                .ok_or_else(|| Error::Blockchain("Не найден последний блок".to_string()))?;
            
            let last_block: BasicBlock = bincode::deserialize(&last_block_data)
                .map_err(|e| Error::Serialization(format!("Не удалось десериализовать последний блок: {}", e)))?;
            
            // Загружаем индекс блоков по высоте
            let mut blocks_by_height = self.blocks_by_height.lock()
                .map_err(|_| Error::Blockchain("Не удалось получить блокировку blocks_by_height".to_string()))?;
            
            for height in 0..=last_height {
                let block_key = format!("block:{}", height).into_bytes();
                if let Some(block_data) = self.storage.get(&block_key).await? {
                    let block: BasicBlock = bincode::deserialize(&block_data)
                        .map_err(|e| Error::Serialization(format!("Не удалось десериализовать блок: {}", e)))?;
                    
                    blocks_by_height.insert(height, block.hash());
                }
            }
            
            // Устанавливаем последний блок
            let mut last_block_lock = self.last_block.lock()
                .map_err(|_| Error::Blockchain("Не удалось получить блокировку last_block".to_string()))?;
            *last_block_lock = Some(last_block);
        } else {
            // Создаем генезис-блок
            let genesis = BasicBlock::genesis();
            
            // Сохраняем генезис-блок
            let genesis_data = bincode::serialize(&genesis)
                .map_err(|e| Error::Serialization(format!("Не удалось сериализовать генезис-блок: {}", e)))?;
            
            self.storage.put(&genesis_key, &genesis_data).await?;
            
            // Обновляем индекс блоков по высоте
            let mut blocks_by_height = self.blocks_by_height.lock()
                .map_err(|_| Error::Blockchain("Не удалось получить блокировку blocks_by_height".to_string()))?;
            
            blocks_by_height.insert(0, genesis.hash());
            
            // Сохраняем высоту последнего блока
            let last_height_data = bincode::serialize(&0u64)
                .map_err(|e| Error::Serialization(format!("Не удалось сериализовать высоту последнего блока: {}", e)))?;
            
            self.storage.put(b"last_height", &last_height_data).await?;
            
            // Устанавливаем последний блок
            let mut last_block_lock = self.last_block.lock()
                .map_err(|_| Error::Blockchain("Не удалось получить блокировку last_block".to_string()))?;
            *last_block_lock = Some(genesis);
        }
        
        Ok(())
    }
}

#[async_trait]
impl Blockchain for BasicBlockchain {
    type BlockType = BasicBlock;
    type TransactionType = BasicTransaction;
    
    async fn get_last_block(&self) -> Result<Self::BlockType> {
        let last_block = self.last_block.lock()
            .map_err(|_| Error::Blockchain("Не удалось получить блокировку last_block".to_string()))?;
        
        last_block.clone().ok_or_else(|| Error::Blockchain("Блокчейн не инициализирован".to_string()))
    }
    
    async fn get_block_by_hash(&self, hash: &[u8]) -> Result<Option<Self::BlockType>> {
        let block_key = format!("block_by_hash:{}", hex::encode(hash)).into_bytes();
        
        if let Some(block_data) = self.storage.get(&block_key).await? {
            let block: BasicBlock = bincode::deserialize(&block_data)
                .map_err(|e| Error::Serialization(format!("Не удалось десериализовать блок: {}", e)))?;
            
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }
    
    async fn get_block_by_height(&self, height: u64) -> Result<Option<Self::BlockType>> {
        let block_key = format!("block:{}", height).into_bytes();
        
        if let Some(block_data) = self.storage.get(&block_key).await? {
            let block: BasicBlock = bincode::deserialize(&block_data)
                .map_err(|e| Error::Serialization(format!("Не удалось десериализовать блок: {}", e)))?;
            
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }
    
    async fn add_block(&mut self, block: Self::BlockType) -> Result<()> {
        // Проверяем валидность блока
        if !block.is_valid() {
            return Err(Error::Blockchain("Блок не валиден".to_string()));
        }
        
        // Проверяем, что предыдущий блок существует
        let last_block = self.get_last_block().await?;
        
        if block.previous_hash() != last_block.hash() {
            return Err(Error::Blockchain("Предыдущий хеш блока не соответствует хешу последнего блока".to_string()));
        }
        
        // Проверяем высоту блока
        if block.height() != last_block.height() + 1 {
            return Err(Error::Blockchain("Высота блока не соответствует ожидаемой".to_string()));
        }
        
        // Сериализуем блок
        let block_data = bincode::serialize(&block)
            .map_err(|e| Error::Serialization(format!("Не удалось сериализовать блок: {}", e)))?;
        
        // Сохраняем блок по высоте
        let block_key = format!("block:{}", block.height()).into_bytes();
        self.storage.put(&block_key, &block_data).await?;
        
        // Сохраняем блок по хешу
        let block_hash_key = format!("block_by_hash:{}", hex::encode(block.hash())).into_bytes();
        self.storage.put(&block_hash_key, &block_data).await?;
        
        // Обновляем индекс блоков по высоте
        let mut blocks_by_height = self.blocks_by_height.lock()
            .map_err(|_| Error::Blockchain("Не удалось получить блокировку blocks_by_height".to_string()))?;
        
        blocks_by_height.insert(block.height(), block.hash());
        
        // Обновляем высоту последнего блока
        let last_height_data = bincode::serialize(&block.height())
            .map_err(|e| Error::Serialization(format!("Не удалось сериализовать высоту последнего блока: {}", e)))?;
        
        self.storage.put(b"last_height", &last_height_data).await?;
        
        // Устанавливаем последний блок
        let mut last_block_lock = self.last_block.lock()
            .map_err(|_| Error::Blockchain("Не удалось получить блокировку last_block".to_string()))?;
        *last_block_lock = Some(block);
        
        Ok(())
    }
    
    async fn add_transaction(&mut self, tx: Self::TransactionType) -> Result<()> {
        // Проверяем валидность транзакции
        if !tx.is_valid() {
            return Err(Error::Blockchain("Транзакция не валидна".to_string()));
        }
        
        // Добавляем транзакцию в пул
        let mut pool = self.transaction_pool.lock()
            .map_err(|_| Error::Blockchain("Не удалось получить блокировку пула транзакций".to_string()))?;
        
        pool.insert(tx);
        
        Ok(())
    }
    
    async fn get_transaction(&self, id: &[u8]) -> Result<Option<Self::TransactionType>> {
        // Ищем транзакцию в пуле
        let pool = self.transaction_pool.lock()
            .map_err(|_| Error::Blockchain("Не удалось получить блокировку пула транзакций".to_string()))?;
        
        for tx in pool.iter() {
            if tx.id() == id {
                return Ok(Some(tx.clone()));
            }
        }
        
        // Если не нашли в пуле, ищем в блоках
        // В реальной реализации здесь будет индекс транзакций
        
        Ok(None)
    }
    
    async fn get_transaction_pool(&self) -> Result<Vec<Self::TransactionType>> {
        let pool = self.transaction_pool.lock()
            .map_err(|_| Error::Blockchain("Не удалось получить блокировку пула транзакций".to_string()))?;
        
        Ok(pool.iter().cloned().collect())
    }
    
    async fn is_chain_valid(&self) -> Result<bool> {
        let blocks_by_height = self.blocks_by_height.lock()
            .map_err(|_| Error::Blockchain("Не удалось получить блокировку blocks_by_height".to_string()))?;
        
        let mut previous_hash = Vec::new();
        
        for height in 0..blocks_by_height.len() as u64 {
            let block = self.get_block_by_height(height).await?
                .ok_or_else(|| Error::Blockchain(format!("Не найден блок на высоте {}", height)))?;
            
            // Проверяем валидность блока
            if !block.is_valid() {
                return Ok(false);
            }
            
            // Проверяем связность цепочки
            if height > 0 && block.previous_hash() != previous_hash {
                return Ok(false);
            }
            
            previous_hash = block.hash();
        }
        
        Ok(true)
    }
} 