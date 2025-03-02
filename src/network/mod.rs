pub mod message;
pub mod peer;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, broadcast};
use futures::stream::{Stream, StreamExt};
use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::types::{PeerId, PeerAddress, PeerInfo, TransportType};
use crate::transport::Transport;
use crate::discovery::Discovery;
use crate::dht::Dht;
use self::message::Message;
use self::peer::Peer;

/// Интерфейс сетевого узла
#[async_trait]
pub trait NetworkNode: Send + Sync {
    /// Получить идентификатор узла
    fn peer_id(&self) -> &PeerId;
    
    /// Подключиться к сети
    async fn connect(&mut self) -> Result<()>;
    
    /// Отключиться от сети
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Обнаружить другие узлы в сети
    async fn discover_peers(&mut self) -> Result<Vec<PeerInfo>>;
    
    /// Отправить сообщение конкретному узлу
    async fn send_to(&mut self, peer_id: &PeerId, data: &[u8]) -> Result<()>;
    
    /// Отправить сообщение всем известным узлам
    async fn broadcast(&mut self, data: &[u8]) -> Result<()>;
    
    /// Получить список известных узлов
    fn peers(&self) -> Vec<PeerInfo>;
    
    /// Получить поток входящих сообщений
    fn incoming(&self) -> Box<dyn Stream<Item = Message> + Unpin + Send>;
}

/// Основной узел сети
pub struct Node {
    /// Идентификатор узла
    peer_id: PeerId,
    /// Адрес для прослушивания
    listen_addr: String,
    /// Порт для прослушивания
    port: u16,
    /// Список транспортных протоколов
    transports: HashMap<TransportType, Box<dyn Transport>>,
    /// Список механизмов обнаружения
    discoveries: Vec<Box<dyn Discovery>>,
    /// Распределенная хеш-таблица
    dht: Option<Box<dyn Dht>>,
    /// Известные узлы
    peers: Arc<Mutex<HashMap<PeerId, Peer>>>,
    /// Канал для отправки сообщений
    message_tx: mpsc::Sender<Message>,
    /// Канал для получения сообщений
    message_rx: mpsc::Receiver<Message>,
    /// Широковещательный канал для входящих сообщений
    broadcast_tx: broadcast::Sender<Message>,
    /// Состояние подключения
    connected: bool,
}

impl Node {
    /// Создать новый узел с помощью NodeBuilder
    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }
    
    /// Внутренний метод создания узла
    fn new(
        peer_id: PeerId,
        listen_addr: String,
        port: u16,
        transports: HashMap<TransportType, Box<dyn Transport>>,
        discoveries: Vec<Box<dyn Discovery>>,
        dht: Option<Box<dyn Dht>>,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::channel(100);
        let (broadcast_tx, _) = broadcast::channel(100);
        
        Self {
            peer_id,
            listen_addr,
            port,
            transports,
            discoveries,
            dht,
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
            message_rx,
            broadcast_tx,
            connected: false,
        }
    }
}

#[async_trait]
impl NetworkNode for Node {
    fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }
    
    async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }
        
        // Запускаем все транспортные протоколы
        for transport in self.transports.values_mut() {
            transport.listen(&self.listen_addr, self.port).await?;
        }
        
        self.connected = true;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }
        
        // Останавливаем все транспортные протоколы
        for transport in self.transports.values_mut() {
            transport.close().await?;
        }
        
        self.connected = false;
        Ok(())
    }
    
    async fn discover_peers(&mut self) -> Result<Vec<PeerInfo>> {
        let mut all_peers = Vec::new();
        
        // Запускаем все механизмы обнаружения
        for discovery in &mut self.discoveries {
            let peers = discovery.discover().await?;
            all_peers.extend(peers);
        }
        
        // Если включен DHT, используем его для обнаружения
        if let Some(dht) = &mut self.dht {
            let peers = dht.find_nodes(&self.peer_id).await?;
            all_peers.extend(peers);
        }
        
        // Добавляем найденных пиров в список известных
        let mut peers_lock = self.peers.lock().expect("Не удалось получить блокировку peers");
        for peer_info in &all_peers {
            if !peers_lock.contains_key(&peer_info.id) {
                let peer = Peer::new(peer_info.clone());
                peers_lock.insert(peer_info.id.clone(), peer);
            }
        }
        
        Ok(all_peers)
    }
    
    async fn send_to(&mut self, peer_id: &PeerId, data: &[u8]) -> Result<()> {
        // Находим пира по идентификатору
        let peers_lock = self.peers.lock().expect("Не удалось получить блокировку peers");
        let peer = peers_lock.get(peer_id).ok_or_else(|| Error::Network(format!("Пир не найден: {}", peer_id)))?;
        
        // Создаем сообщение
        let message = Message::new_data(self.peer_id.clone(), peer_id.clone(), data.to_vec());
        
        // Выбираем транспорт для отправки
        // Для простоты используем первый доступный транспорт
        if let Some(transport) = self.transports.values().next() {
            if let Some(addr) = &peer.info().address {
                transport.send_to(addr, &bincode::serialize(&message)?).await?;
                Ok(())
            } else {
                Err(Error::Network(format!("Адрес пира не известен: {}", peer_id)))
            }
        } else {
            Err(Error::Network("Нет доступных транспортных протоколов".to_string()))
        }
    }
    
    async fn broadcast(&mut self, data: &[u8]) -> Result<()> {
        let peers_lock = self.peers.lock().expect("Не удалось получить блокировку peers");
        let peer_ids: Vec<PeerId> = peers_lock.keys().cloned().collect();
        drop(peers_lock);
        
        for peer_id in peer_ids {
            // Игнорируем ошибки при отправке отдельным узлам
            let _ = self.send_to(&peer_id, data).await;
        }
        
        Ok(())
    }
    
    fn peers(&self) -> Vec<PeerInfo> {
        let peers_lock = self.peers.lock().expect("Не удалось получить блокировку peers");
        peers_lock.values().map(|p| p.info().clone()).collect()
    }
    
    fn incoming(&self) -> Box<dyn Stream<Item = Message> + Unpin + Send> {
        let rx = self.broadcast_tx.subscribe();
        Box::new(tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(|r| async move { r.ok() }))
    }
}

/// Строитель для настройки и создания узла сети
pub struct NodeBuilder {
    listen_addr: String,
    port: u16,
    transports: HashMap<TransportType, Box<dyn Transport>>,
    discoveries: Vec<Box<dyn Discovery>>,
    dht: Option<Box<dyn Dht>>,
    peer_id: Option<PeerId>,
}

impl NodeBuilder {
    /// Создать новый строитель узла
    pub fn new() -> Self {
        Self {
            listen_addr: "127.0.0.1".to_string(),
            port: 0, // Случайный порт
            transports: HashMap::new(),
            discoveries: Vec::new(),
            dht: None,
            peer_id: None,
        }
    }
    
    /// Установить адрес для прослушивания
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.listen_addr = address.into();
        self
    }
    
    /// Установить порт для прослушивания
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    /// Добавить транспортный протокол
    pub fn with_transport(mut self, transport_type: TransportType, transport: Box<dyn Transport>) -> Self {
        self.transports.insert(transport_type, transport);
        self
    }
    
    /// Добавить механизм обнаружения узлов
    pub fn with_discovery(mut self, discovery: Box<dyn Discovery>) -> Self {
        self.discoveries.push(discovery);
        self
    }
    
    /// Добавить поддержку mDNS для локального обнаружения
    pub fn with_mdns(self) -> Self {
        // Реализация будет добавлена в модуле discovery
        // Это просто заглушка для интерфейса, описанного в README
        self
    }
    
    /// Добавить распределенную хеш-таблицу
    pub fn with_dht(self) -> Self {
        // Реализация будет добавлена в модуле dht
        // Это просто заглушка для интерфейса, описанного в README
        self
    }
    
    /// Установить идентификатор узла
    pub fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }
    
    /// Создать узел с заданными параметрами
    pub fn build(self) -> Result<Node> {
        // Если идентификатор не указан, генерируем случайный
        let peer_id = self.peer_id.unwrap_or_else(|| {
            // Генерируем случайный ID
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
            PeerId::new(bytes)
        });
        
        let node = Node::new(
            peer_id,
            self.listen_addr,
            self.port,
            self.transports,
            self.discoveries,
            self.dht,
        );
        
        Ok(node)
    }
} 