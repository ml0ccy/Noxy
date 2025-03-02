use async_trait::async_trait;
use crate::error::Result;
use crate::types::PeerInfo;

/// Трейт для механизмов обнаружения узлов
#[async_trait]
pub trait Discovery: Send + Sync {
    /// Получить имя механизма обнаружения
    fn name(&self) -> &str;
    
    /// Запустить процесс обнаружения
    async fn start(&mut self) -> Result<()>;
    
    /// Остановить процесс обнаружения
    async fn stop(&mut self) -> Result<()>;
    
    /// Обнаружить узлы в сети
    async fn discover(&mut self) -> Result<Vec<PeerInfo>>;
}

pub mod mdns;
pub mod bootstrap; 