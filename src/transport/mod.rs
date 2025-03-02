use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use crate::error::Result;
use crate::types::TransportType;

/// Трейт для транспортных протоколов
#[async_trait]
pub trait Transport: Send + Sync {
    /// Получить тип транспорта
    fn transport_type(&self) -> TransportType;
    
    /// Начать прослушивание входящих соединений
    async fn listen(&mut self, address: &str, port: u16) -> Result<()>;
    
    /// Подключиться к удаленному узлу
    async fn connect(&mut self, address: &str) -> Result<()>;
    
    /// Отправить данные на указанный адрес
    async fn send_to(&self, address: &str, data: &[u8]) -> Result<()>;
    
    /// Получить канал для входящих сообщений
    fn incoming(&self) -> mpsc::Receiver<(Vec<u8>, SocketAddr)>;
    
    /// Закрыть все соединения
    async fn close(&mut self) -> Result<()>;
}

pub mod tcp;
pub mod websocket; 