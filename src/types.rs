use std::fmt;
use serde::{Serialize, Deserialize};

/// Идентификатор узла в сети
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(Vec<u8>);

impl PeerId {
    /// Создать новый PeerId из байтов
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Получить байтовое представление
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

/// Адрес узла в сети
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerAddress {
    /// IP адрес и порт
    pub address: String,
    /// Идентификатор узла
    pub peer_id: PeerId,
}

impl PeerAddress {
    /// Создать новый адрес узла
    pub fn new(address: String, peer_id: PeerId) -> Self {
        Self { address, peer_id }
    }
}

impl fmt::Display for PeerAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.peer_id, self.address)
    }
}

/// Метаданные узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Идентификатор узла
    pub id: PeerId,
    /// Адрес узла
    pub address: Option<String>,
    /// Поддерживаемые протоколы
    pub protocols: Vec<String>,
    /// Версия клиента
    pub client_version: String,
}

/// Тип протокола транспортного уровня
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportType {
    /// TCP протокол
    Tcp,
    /// WebSocket протокол
    WebSocket,
    /// Пользовательский транспорт
    Custom,
} 