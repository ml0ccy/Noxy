use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::types::PeerId;

/// Типы сообщений
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Сообщение с данными
    Data,
    /// Запрос на поиск узла
    FindNode,
    /// Ответ на поиск узла
    NodeResponse,
    /// Ping для проверки соединения
    Ping,
    /// Pong в ответ на ping
    Pong,
    /// Объявление о присутствии
    Announce,
    /// Запрос на хранение данных
    Store,
    /// Запрос на получение данных
    Get,
    /// Ответ с данными
    Value,
    /// Пользовательский тип сообщения
    Custom(u8),
}

/// Сетевое сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Идентификатор отправителя
    pub from: PeerId,
    /// Идентификатор получателя (если None, то broadcast)
    pub to: Option<PeerId>,
    /// Тип сообщения
    pub message_type: MessageType,
    /// Данные сообщения
    pub data: Vec<u8>,
    /// Временная метка отправки
    pub timestamp: u64,
    /// Уникальный идентификатор сообщения
    pub id: [u8; 16],
}

impl Message {
    /// Создать новое сообщение с данными
    pub fn new_data(from: PeerId, to: PeerId, data: Vec<u8>) -> Self {
        Self::new(from, Some(to), MessageType::Data, data)
    }
    
    /// Создать новое широковещательное сообщение с данными
    pub fn new_broadcast(from: PeerId, data: Vec<u8>) -> Self {
        Self::new(from, None, MessageType::Data, data)
    }
    
    /// Создать новое сообщение
    pub fn new(from: PeerId, to: Option<PeerId>, message_type: MessageType, data: Vec<u8>) -> Self {
        // Получаем текущее время в миллисекундах
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Время до начала эпохи")
            .as_millis() as u64;
        
        // Генерируем случайный ID
        let mut id = [0u8; 16];
        rand::Rng::fill(&mut rand::thread_rng(), &mut id);
        
        Self {
            from,
            to,
            message_type,
            data,
            timestamp,
            id,
        }
    }
    
    /// Создать ответ на это сообщение
    pub fn create_response(&self, response_type: MessageType, data: Vec<u8>) -> Self {
        Self::new(
            self.to.clone().expect("Сообщение должно иметь получателя"),
            Some(self.from.clone()),
            response_type,
            data,
        )
    }
} 