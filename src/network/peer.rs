use std::time::{Duration, Instant};
use crate::types::PeerInfo;

/// Статус подключения к пиру
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerStatus {
    /// Не подключен
    Disconnected,
    /// В процессе подключения
    Connecting,
    /// Подключен
    Connected,
    /// Неизвестный статус
    Unknown,
}

/// Представление пира в сети
pub struct Peer {
    /// Информация о пире
    info: PeerInfo,
    /// Статус подключения
    status: PeerStatus,
    /// Время последнего контакта с пиром
    last_seen: Instant,
    /// Время первого контакта с пиром
    first_seen: Instant,
    /// Счетчик неудачных попыток подключения
    failed_attempts: u32,
}

impl Peer {
    /// Создать нового пира
    pub fn new(info: PeerInfo) -> Self {
        let now = Instant::now();
        Self {
            info,
            status: PeerStatus::Disconnected,
            last_seen: now,
            first_seen: now,
            failed_attempts: 0,
        }
    }
    
    /// Получить информацию о пире
    pub fn info(&self) -> &PeerInfo {
        &self.info
    }
    
    /// Получить текущий статус пира
    pub fn status(&self) -> PeerStatus {
        self.status
    }
    
    /// Установить статус подключения
    pub fn set_status(&mut self, status: PeerStatus) {
        self.status = status;
        if status == PeerStatus::Connected {
            self.failed_attempts = 0;
        }
    }
    
    /// Обновить время последнего контакта
    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }
    
    /// Получить время с момента последнего контакта
    pub fn time_since_last_seen(&self) -> Duration {
        self.last_seen.elapsed()
    }
    
    /// Увеличить счетчик неудачных попыток
    pub fn increment_failed_attempts(&mut self) {
        self.failed_attempts += 1;
    }
    
    /// Получить количество неудачных попыток
    pub fn failed_attempts(&self) -> u32 {
        self.failed_attempts
    }
    
    /// Проверить, устарел ли пир (давно не было контакта)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.time_since_last_seen() > timeout
    }
} 