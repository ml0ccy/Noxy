//! # noxy
//! 
//! A Rust library for building P2P (peer-to-peer) decentralized applications, including blockchain functionality.
//!
//! ## P2P Network Example
//!
//! ```rust,no_run
//! use noxy::prelude::*;
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new network node
//!     let mut node = NodeBuilder::new()
//!         .with_port(8000)
//!         .with_dht()
//!         .with_mdns()
//!         .build()?;
//!     
//!     // Connect to the network
//!     node.connect().await?;
//!     
//!     // Discover other nodes
//!     node.discover_peers().await?;
//!     
//!     // Broadcast a message to all nodes
//!     node.broadcast("Hello, P2P world!".as_bytes()).await?;
//!     
//!     // Process incoming messages
//!     let mut incoming = node.incoming();
//!     while let Some(message) = incoming.next().await {
//!         println!("Received: {:?}", message);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Blockchain Example
//!
//! ```rust,no_run
//! use noxy::prelude::*;
//! use noxy::blockchain::basic::{BasicBlock, BasicTransaction, BasicBlockchain};
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a key pair
//!     let keypair = crypto::generate_ed25519_keypair()?;
//!     let pubkey = keypair.public_key();
//!     
//!     // Create a blockchain with difficulty level 2
//!     let mut blockchain = BasicBlockchain::new(2).await?;
//!     
//!     // Create and sign a transaction
//!     let mut tx = BasicTransaction::new(
//!         pubkey.clone(), 
//!         vec![0; 32], // Receiver
//!         10.0, 
//!         "Test transaction".to_string()
//!     );
//!     tx.sign(&keypair)?;
//!     
//!     // Add transaction to the pool
//!     blockchain.add_transaction(Box::new(tx)).await?;
//!     
//!     // Mine a new block
//!     let genesis = blockchain.get_last_block().await?;
//!     let mut new_block = BasicBlock::new(
//!         genesis.hash().to_vec(),
//!         genesis.height() + 1,
//!         blockchain.get_difficulty(),
//!         blockchain.get_pending_transactions().await,
//!         "Block data".to_string()
//!     );
//!     
//!     // Mine the block and add it to the chain
//!     new_block.mine();
//!     blockchain.add_block(Box::new(new_block)).await?;
//!     
//!     Ok(())
//! }
//! ```

/// Library errors
pub mod error;

/// Core network components
pub mod network;

/// Node discovery mechanisms
pub mod discovery;

/// Transport protocols
pub mod transport;

/// Distributed hash table
pub mod dht;

/// Cryptographic primitives
pub mod crypto;

/// Blockchain components
pub mod blockchain;

/// Data storage
pub mod storage;

/// Common data types and utilities
pub mod types;

/// Re-exports of main components for convenience
pub mod prelude {
    pub use crate::network::{Node, NodeBuilder};
    pub use crate::network::message::Message;
    pub use crate::error::Error;
    pub use crate::types::PeerId;
    
    // Blockchain components
    pub use crate::blockchain::{Block, Transaction, Blockchain};
    
    // Cryptographic components
    pub use crate::crypto;
    
    // Transport types
    pub use crate::types::TransportType;
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");