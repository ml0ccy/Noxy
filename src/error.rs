use thiserror::Error;
use std::io;

/// Типы ошибок, которые могут возникнуть в библиотеке
#[derive(Error, Debug)]
pub enum Error {
    /// Ошибка ввода/вывода
    #[error("Ошибка ввода/вывода: {0}")]
    Io(#[from] io::Error),

    /// Ошибка сети
    #[error("Ошибка сети: {0}")]
    Network(String),

    /// Ошибка DHT
    #[error("Ошибка DHT: {0}")]
    Dht(String),

    /// Ошибка обнаружения узлов
    #[error("Ошибка обнаружения узлов: {0}")]
    Discovery(String),

    /// Ошибка транспортного уровня
    #[error("Ошибка транспорта: {0}")]
    Transport(String),

    /// Ошибка криптографии
    #[error("Ошибка криптографии: {0}")]
    Crypto(String),

    /// Ошибка сериализации/десериализации
    #[error("Ошибка сериализации: {0}")]
    Serialization(String),

    /// Ошибка блокчейна
    #[error("Ошибка блокчейна: {0}")]
    Blockchain(String),

    /// Ошибка хранилища
    #[error("Ошибка хранилища: {0}")]
    Storage(String),

    /// Неизвестная ошибка
    #[error("Неизвестная ошибка: {0}")]
    Unknown(String),
}

/// Расширение для Result с нашим типом ошибки
pub type Result<T> = std::result::Result<T, Error>; 