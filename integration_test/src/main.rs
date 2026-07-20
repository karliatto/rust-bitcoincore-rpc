//! # rust-bitcoincore-rpc integration test
//!
//! Exercises the ContextVM (MCP-over-Nostr) client against a running bitcoind
//! RPC server that is exposed through a ContextVM MCP server.
//!
//! The goal is not to test the correctness of the node, but to exercise the
//! serialization of arguments and deserialization of responses for the subset
//! of RPC methods this client supports.
//!
//! Configuration is taken from CLI arguments or environment variables:
//!
//! ```text
//! integration_test <server_pubkey> [relay_url]
//! ```
//!
//! or via `SERVER_PUBKEY` and `RELAY_URL`.

use bitcoincore_rpc::{bitcoin, Client, Error, RpcApi};

struct StdLogger;

impl log::Log for StdLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().contains("bitcoincore_rpc")
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("[{}][{}]: {}", record.level(), record.metadata().target(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: StdLogger = StdLogger;

fn main_result() -> Result<(), Error> {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    let mut args = std::env::args().skip(1);

    let server_pubkey = args
        .next()
        .or_else(|| std::env::var("SERVER_PUBKEY").ok())
        .expect("provide the server pubkey as the first argument or via SERVER_PUBKEY");
    let relay_url = args
        .next()
        .or_else(|| std::env::var("RELAY_URL").ok())
        .unwrap_or_else(|| "ws://localhost:10547".to_string());

    println!("Relay URL: {}", relay_url);
    println!("Server public key: {}", server_pubkey);

    let rpc = Client::new(vec![relay_url], server_pubkey)?;

    // Diagnostic: dump the server's advertised tools and their input schemas.
    rpc.dump_tool_schemas()?;

    test_get_blockchain_info(&rpc);
    let tip_hash = test_get_block_count_and_hash(&rpc);
    test_get_block_header_info(&rpc, &tip_hash);
    let block = test_get_block(&rpc, &tip_hash);
    test_get_block_info(&rpc, &tip_hash);
    test_get_raw_mempool(&rpc);
    test_get_raw_transaction(&rpc, &tip_hash, &block);
    test_get_by_id(&rpc, &tip_hash);

    println!("integration test completed successfully");
    Ok(())
}

fn test_get_blockchain_info(rpc: &Client) {
    let info = rpc.get_blockchain_info().unwrap();
    println!("blockchain: chain={} blocks={}", info.chain, info.blocks);
    let version = rpc.version().unwrap();
    println!("server version: {}", version);
}

fn test_get_block_count_and_hash(rpc: &Client) -> bitcoin::BlockHash {
    let count = rpc.get_block_count().unwrap();
    println!("block count: {}", count);
    let tip_hash = rpc.get_block_hash(count).unwrap();
    println!("tip hash: {}", tip_hash);
    tip_hash
}

fn test_get_block_header_info(rpc: &Client, tip_hash: &bitcoin::BlockHash) {
    let header = rpc.get_block_header_info(tip_hash).unwrap();
    println!("tip header height: {}", header.height);
    assert_eq!(&header.hash, tip_hash);
}

fn test_get_block(rpc: &Client, tip_hash: &bitcoin::BlockHash) -> bitcoin::Block {
    let block = rpc.get_block(tip_hash).unwrap();
    println!("tip block tx count: {}", block.txdata.len());
    assert_eq!(&block.block_hash(), tip_hash);
    block
}

fn test_get_block_info(rpc: &Client, tip_hash: &bitcoin::BlockHash) {
    let block_info = rpc.get_block_info(tip_hash).unwrap();
    println!("tip block (verbose) size: {}", block_info.size);
}

fn test_get_raw_mempool(rpc: &Client) {
    let mempool = rpc.get_raw_mempool().unwrap();
    println!("mempool tx count: {}", mempool.len());
}

fn test_get_raw_transaction(rpc: &Client, tip_hash: &bitcoin::BlockHash, block: &bitcoin::Block) {
    if let Some(coinbase) = block.txdata.first() {
        let txid = coinbase.compute_txid();
        match rpc.get_raw_transaction(&txid, Some(tip_hash)) {
            Ok(tx) => println!("coinbase txid: {}", tx.compute_txid()),
            Err(e) => println!("get_raw_transaction failed (expected without -txindex): {}", e),
        }
    }
}

fn test_get_by_id(rpc: &Client, tip_hash: &bitcoin::BlockHash) {
    let _block: bitcoin::Block = rpc.get_by_id(tip_hash).unwrap();
}

fn main() {
    main_result().unwrap();
}
