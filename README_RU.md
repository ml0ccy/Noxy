# Noxy

Библиотека для создания P2P (peer-to-peer) децентрализованных приложений на Rust, включая блокчейны.

## Возможности

- **P2P сеть**: Создание и управление узлами в децентрализованной сети
- **Транспортные протоколы**: Поддержка TCP и других протоколов
- **Обнаружение узлов**: mDNS и другие механизмы обнаружения
- **DHT**: Распределенная хеш-таблица на основе Kademlia
- **Криптография**: Подписи Ed25519, хеширование и другие криптографические примитивы
- **Блокчейн**: Компоненты для создания блокчейн-приложений
- **Хранение**: Различные механизмы хранения данных

## Установка

Добавьте в Cargo.toml:

```toml
[dependencies]
noxy = "0.1.0"
```

## Примеры использования

### P2P сеть

```rust
use noxy::prelude::*;
use noxy::transport::tcp::TcpTransport;
use noxy::types::TransportType;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создание нового узла сети
    let mut node = NodeBuilder::new()
        .with_port(8000)
        .with_transport(
            TransportType::Tcp,
            Box::new(TcpTransport::new())
        )
        .build()?;
    
    // Подключение к сети
    node.connect().await?;
    
    // Обнаружение других узлов
    let peers = node.discover_peers().await?;
    println!("Найдено узлов: {}", peers.len());
    
    // Отправка сообщения всем узлам
    node.broadcast("Hello, P2P world!".as_bytes()).await?;
    
    Ok(())
}
```

### Блокчейн

```rust
use noxy::prelude::*;
use noxy::blockchain::basic::{BasicBlock, BasicTransaction, BasicBlockchain};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаем ключевую пару
    let keypair = crypto::generate_ed25519_keypair()?;
    let pubkey = keypair.public_key();
    
    // Создаем блокчейн с уровнем сложности 2
    let mut blockchain = BasicBlockchain::new(2).await?;
    
    // Создаем и подписываем транзакцию
    let mut tx = BasicTransaction::new(
        pubkey.clone(), 
        vec![0; 32], // Получатель
        10.0, 
        "Тестовая транзакция".to_string()
    );
    tx.sign(&keypair)?;
    
    // Добавляем транзакцию в пул
    blockchain.add_transaction(Box::new(tx)).await?;
    
    // Майним новый блок
    let genesis = blockchain.get_last_block().await?;
    let mut new_block = BasicBlock::new(
        genesis.hash().to_vec(),
        genesis.height() + 1,
        blockchain.get_difficulty(),
        blockchain.get_pending_transactions().await,
        "Данные блока".to_string()
    );
    
    // Майним блок и добавляем в цепочку
    new_block.mine();
    blockchain.add_block(Box::new(new_block)).await?;
    
    Ok(())
}
```

## Запуск примеров

```bash
# Запуск простого P2P узла
cargo run --example simple

# Запуск примера блокчейна
cargo run --example blockchain
```

## Лицензия

MIT 