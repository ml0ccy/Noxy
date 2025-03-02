use noxy::prelude::*;
use noxy::blockchain::basic::{BasicBlock, BasicTransaction, BasicBlockchain};
use noxy::blockchain::{Block, Transaction, Blockchain};
use noxy::crypto;
use noxy::types::PeerId;

use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Инициализируем журналирование
    tracing_subscriber::fmt::init();
    
    println!("Демонстрация блокчейна noxy v{}", noxy::VERSION);
    
    // Создаем ключевые пары для участников
    let alice_keypair = crypto::generate_ed25519_keypair()?;
    let bob_keypair = crypto::generate_ed25519_keypair()?;
    let charlie_keypair = crypto::generate_ed25519_keypair()?;
    
    // Получаем публичные ключи (адреса) участников
    let alice_pubkey = alice_keypair.public_key();
    let bob_pubkey = bob_keypair.public_key();
    let charlie_pubkey = charlie_keypair.public_key();
    
    println!("Участники:");
    println!("Alice: {}", hex::encode(&alice_pubkey));
    println!("Bob: {}", hex::encode(&bob_pubkey));
    println!("Charlie: {}", hex::encode(&charlie_pubkey));
    
    // Создаем блокчейн с уровнем сложности 2 (для демонстрации)
    let mut blockchain = BasicBlockchain::new(2).await?;
    println!("Создан блокчейн с уровнем сложности 2");
    
    // Получаем последний блок (генезис)
    let genesis = blockchain.get_last_block().await?;
    println!("Генезис блок: {}", hex::encode(genesis.hash()));
    
    // Создаем и подписываем транзакции
    println!("\nСоздание транзакций...");
    
    // Alice отправляет 50 монет Bob
    let mut tx1 = BasicTransaction::new(
        alice_pubkey.clone(), 
        bob_pubkey.clone(), 
        50.0, 
        "Первая транзакция".to_string()
    );
    tx1.sign(&alice_keypair)?;
    println!("Транзакция 1: {} -> {} (50 монет)", 
        hex::encode(&alice_pubkey)[..8], 
        hex::encode(&bob_pubkey)[..8]
    );
    
    // Bob отправляет 20 монет Charlie
    let mut tx2 = BasicTransaction::new(
        bob_pubkey.clone(), 
        charlie_pubkey.clone(), 
        20.0, 
        "Вторая транзакция".to_string()
    );
    tx2.sign(&bob_keypair)?;
    println!("Транзакция 2: {} -> {} (20 монет)", 
        hex::encode(&bob_pubkey)[..8], 
        hex::encode(&charlie_pubkey)[..8]
    );
    
    // Charlie отправляет 5 монет Alice
    let mut tx3 = BasicTransaction::new(
        charlie_pubkey.clone(), 
        alice_pubkey.clone(), 
        5.0, 
        "Третья транзакция".to_string()
    );
    tx3.sign(&charlie_keypair)?;
    println!("Транзакция 3: {} -> {} (5 монет)", 
        hex::encode(&charlie_pubkey)[..8], 
        hex::encode(&alice_pubkey)[..8]
    );
    
    // Добавляем транзакции в пул
    blockchain.add_transaction(Box::new(tx1)).await?;
    blockchain.add_transaction(Box::new(tx2)).await?;
    blockchain.add_transaction(Box::new(tx3)).await?;
    
    println!("\nДобавлено 3 транзакции в пул");
    
    // Майним новый блок
    println!("\nМайнинг блока...");
    let start = std::time::Instant::now();
    
    // Создаем новый блок с данными
    let mut new_block = BasicBlock::new(
        genesis.hash().to_vec(),
        genesis.height() + 1,
        blockchain.get_difficulty(),
        blockchain.get_pending_transactions().await,
        "Данные блока #1".to_string()
    );
    
    // Майним блок (находим подходящий nonce)
    new_block.mine();
    
    let duration = start.elapsed();
    println!("Блок найден за {:?}!", duration);
    println!("Хеш: {}", hex::encode(new_block.hash()));
    println!("Nonce: {}", new_block.get_nonce());
    
    // Добавляем блок в цепочку
    blockchain.add_block(Box::new(new_block)).await?;
    println!("\nБлок успешно добавлен в цепочку");
    
    // Проверяем валидность цепочки
    let is_valid = blockchain.is_chain_valid().await?;
    println!("Проверка валидности цепочки: {}", if is_valid { "Успешно" } else { "Ошибка" });
    
    // Выводим информацию о блокчейне
    println!("\nИнформация о блокчейне:");
    println!("Количество блоков: {}", blockchain.get_chain_length().await);
    println!("Последний блок: {}", hex::encode(blockchain.get_last_block().await?.hash()));
    
    Ok(())
} 