use noxy::prelude::*;
use noxy::transport::tcp::TcpTransport;
use noxy::discovery::mdns::MdnsDiscovery;
use noxy::types::{PeerId, TransportType};
use noxy::crypto;

use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Инициализируем журналирование
    tracing_subscriber::fmt::init();
    
    println!("Запуск простого P2P узла на noxy v{}", noxy::VERSION);
    
    // Создаем случайный идентификатор для узла
    let peer_id = PeerId::new(rand::random::<[u8; 32]>().to_vec());
    
    // Создаем ключи
    let key_pair = crypto::generate_ed25519_keypair()?;
    
    // Создаем и настраиваем узел
    let mut node = NodeBuilder::new()
        .with_peer_id(peer_id.clone())
        .with_port(8000)
        .with_transport(
            TransportType::Tcp,
            Box::new(TcpTransport::new())
        )
        .build()?;
    
    println!("Создан узел с ID: {}", peer_id);
    
    // Подключаемся к сети
    println!("Подключение к сети...");
    node.connect().await?;
    println!("Успешно подключен к сети");
    
    // Обнаруживаем других участников
    println!("Поиск других узлов...");
    let peers = node.discover_peers().await?;
    println!("Найдено узлов: {}", peers.len());
    
    for (i, peer) in peers.iter().enumerate() {
        println!("Пир {}: {}", i + 1, peer.id);
    }
    
    // Отправляем тестовое сообщение всем известным узлам
    println!("Отправка тестового сообщения...");
    node.broadcast("Привет от noxy!".as_bytes()).await?;
    
    // Обрабатываем входящие сообщения в течение 30 секунд
    println!("Ожидание входящих сообщений в течение 30 секунд...");
    let mut incoming = node.incoming();
    
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
            println!("Время ожидания истекло");
        },
        result = async {
            while let Some(message) = incoming.next().await {
                println!(
                    "Получено сообщение от {}: {}",
                    message.from,
                    String::from_utf8_lossy(&message.data)
                );
            }
            Ok::<_, Box<dyn Error>>(())
        } => {
            result?;
        }
    }
    
    // Отключаемся от сети
    println!("Отключение от сети...");
    node.disconnect().await?;
    println!("Успешно отключен от сети");
    
    Ok(())
} 