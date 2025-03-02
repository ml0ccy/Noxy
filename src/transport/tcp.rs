use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::error::{Error, Result};
use crate::types::TransportType;
use super::Transport;

/// Реализация транспорта на основе TCP
pub struct TcpTransport {
    /// Канал для отправки входящих сообщений
    incoming_tx: mpsc::Sender<(Vec<u8>, SocketAddr)>,
    /// Канал для получения входящих сообщений
    incoming_rx: mpsc::Receiver<(Vec<u8>, SocketAddr)>,
    /// Активные соединения
    connections: Arc<Mutex<HashMap<String, TcpStream>>>,
    /// Задача для прослушивания входящих соединений
    listener_task: Option<JoinHandle<()>>,
    /// Адрес для прослушивания
    listen_addr: Option<SocketAddr>,
    /// Размер буфера для чтения
    read_buffer_size: usize,
}

impl TcpTransport {
    /// Создать новый TCP транспорт
    pub fn new() -> Self {
        let (incoming_tx, incoming_rx) = mpsc::channel(100);
        
        Self {
            incoming_tx,
            incoming_rx,
            connections: Arc::new(Mutex::new(HashMap::new())),
            listener_task: None,
            listen_addr: None,
            read_buffer_size: 4096, // 4 KB
        }
    }
    
    /// Установить размер буфера для чтения
    pub fn with_read_buffer_size(mut self, size: usize) -> Self {
        self.read_buffer_size = size;
        self
    }
    
    /// Обработать входящее соединение
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        tx: mpsc::Sender<(Vec<u8>, SocketAddr)>,
        buffer_size: usize,
    ) {
        let mut stream = stream;
        let mut buffer = vec![0u8; buffer_size];
        
        // Читаем данные из соединения
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    // Соединение закрыто
                    break;
                }
                Ok(n) => {
                    // Отправляем данные в канал
                    if let Err(_) = tx.send((buffer[..n].to_vec(), addr)).await {
                        // Канал закрыт, выходим из цикла
                        break;
                    }
                }
                Err(_) => {
                    // Ошибка чтения, выходим из цикла
                    break;
                }
            }
        }
    }
}

#[async_trait]
impl Transport for TcpTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }
    
    async fn listen(&mut self, address: &str, port: u16) -> Result<()> {
        // Создаем адрес для прослушивания
        let addr = format!("{}:{}", address, port).parse::<SocketAddr>()
            .map_err(|e| Error::Transport(format!("Неверный адрес: {}", e)))?;
        
        // Создаем TCP слушателя
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| Error::Transport(format!("Не удалось привязаться к адресу {}: {}", addr, e)))?;
        
        let connections = Arc::clone(&self.connections);
        let tx = self.incoming_tx.clone();
        let buffer_size = self.read_buffer_size;
        
        // Запускаем задачу для прослушивания
        let task = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        // Сохраняем соединение
                        let addr_str = addr.to_string();
                        let mut stream_for_map = stream.try_clone().unwrap();
                        connections.lock().unwrap().insert(addr_str, stream_for_map);
                        
                        // Запускаем обработку соединения
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            Self::handle_connection(stream, addr, tx_clone, buffer_size).await;
                        });
                    }
                    Err(_) => {
                        // Ошибка при принятии соединения
                        continue;
                    }
                }
            }
        });
        
        self.listener_task = Some(task);
        self.listen_addr = Some(addr);
        
        Ok(())
    }
    
    async fn connect(&mut self, address: &str) -> Result<()> {
        // Подключаемся к удаленному адресу
        let stream = TcpStream::connect(address).await
            .map_err(|e| Error::Transport(format!("Не удалось подключиться к {}: {}", address, e)))?;
        
        // Сохраняем соединение
        let mut connections = self.connections.lock().unwrap();
        connections.insert(address.to_string(), stream);
        
        Ok(())
    }
    
    async fn send_to(&self, address: &str, data: &[u8]) -> Result<()> {
        // Проверяем, есть ли соединение
        let mut connections = self.connections.lock().unwrap();
        let stream = if let Some(stream) = connections.get_mut(address) {
            stream
        } else {
            // Если нет соединения, пытаемся подключиться
            let stream = TcpStream::connect(address).await
                .map_err(|e| Error::Transport(format!("Не удалось подключиться к {}: {}", address, e)))?;
            connections.insert(address.to_string(), stream);
            connections.get_mut(address).unwrap()
        };
        
        // Отправляем данные
        stream.write_all(data).await
            .map_err(|e| Error::Transport(format!("Ошибка отправки данных: {}", e)))?;
        
        Ok(())
    }
    
    fn incoming(&self) -> mpsc::Receiver<(Vec<u8>, SocketAddr)> {
        self.incoming_rx.clone()
    }
    
    async fn close(&mut self) -> Result<()> {
        // Отменяем задачу прослушивания
        if let Some(task) = self.listener_task.take() {
            task.abort();
        }
        
        // Закрываем все соединения
        let mut connections = self.connections.lock().unwrap();
        connections.clear();
        
        Ok(())
    }
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new()
    }
} 