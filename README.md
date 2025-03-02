# Noxy

A Rust library for building P2P (peer-to-peer) decentralized applications, including blockchain functionality.

## Features

- **P2P Network**: Create and manage nodes in a decentralized network
- **Transport Protocols**: Support for TCP and other protocols
- **Node Discovery**: mDNS and other discovery mechanisms
- **DHT**: Kademlia-based distributed hash table
- **Cryptography**: Ed25519 signatures, hashing, and other cryptographic primitives
- **Blockchain**: Components for building blockchain applications
- **Storage**: Various data storage mechanisms

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
noxy = "0.1.0"
```

## Usage Examples

### P2P Network

```rust
use noxy::prelude::*;
use noxy::transport::tcp::TcpTransport;
use noxy::types::TransportType;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new network node
    let mut node = NodeBuilder::new()
        .with_port(8000)
        .with_transport(
            TransportType::Tcp,
            Box::new(TcpTransport::new())
        )
        .build()?;
    
    // Connect to the network
    node.connect().await?;
    
    // Discover other nodes
    let peers = node.discover_peers().await?;
    println!("Nodes found: {}", peers.len());
    
    // Broadcast a message to all nodes
    node.broadcast("Hello, P2P world!".as_bytes()).await?;
    
    Ok(())
}
```

### Blockchain

```rust
use noxy::prelude::*;
use noxy::blockchain::basic::{BasicBlock, BasicTransaction, BasicBlockchain};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a key pair
    let keypair = crypto::generate_ed25519_keypair()?;
    let pubkey = keypair.public_key();
    
    // Create a blockchain with difficulty level 2
    let mut blockchain = BasicBlockchain::new(2).await?;
    
    // Create and sign a transaction
    let mut tx = BasicTransaction::new(
        pubkey.clone(), 
        vec![0; 32], // Receiver
        10.0, 
        "Test transaction".to_string()
    );
    tx.sign(&keypair)?;
    
    // Add transaction to the pool
    blockchain.add_transaction(Box::new(tx)).await?;
    
    // Mine a new block
    let genesis = blockchain.get_last_block().await?;
    let mut new_block = BasicBlock::new(
        genesis.hash().to_vec(),
        genesis.height() + 1,
        blockchain.get_difficulty(),
        blockchain.get_pending_transactions().await,
        "Block data".to_string()
    );
    
    // Mine the block and add it to the chain
    new_block.mine();
    blockchain.add_block(Box::new(new_block)).await?;
    
    Ok(())
}
```

## Running Examples

```bash
# Run the simple P2P node example
cargo run --example simple

# Run the blockchain example
cargo run --example blockchain
```

## License

MIT
