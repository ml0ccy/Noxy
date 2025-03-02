use async_trait::async_trait;
use crate::error::Result;
use crate::types::{PeerId, PeerInfo};

/// Трейт для распределенной хеш-таблицы
#[async_trait]
pub trait Dht: Send + Sync {
    /// Начать прослушивание DHT сети
    async fn start(&mut self) -> Result<()>;
    
    /// Остановить прослушивание DHT сети
    async fn stop(&mut self) -> Result<()>;
    
    /// Найти узлы в сети
    async fn find_nodes(&mut self, target: &PeerId) -> Result<Vec<PeerInfo>>;
    
    /// Найти значение по ключу
    async fn find_value(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// Сохранить значение по ключу
    async fn store(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
    
    /// Добавить узел в таблицу маршрутизации
    async fn add_peer(&mut self, peer: PeerInfo) -> Result<()>;
    
    /// Получить ближайшие узлы к заданному ID
    async fn get_closest_peers(&mut self, target: &PeerId, limit: usize) -> Result<Vec<PeerInfo>>;
}

pub mod kademlia; 