// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! A very simple example used as a self-test of this library against a Bitcoin
//! Core node exposed via a ContextVM MCP-over-Nostr server.
extern crate bitcoincore_rpc;

use bitcoincore_rpc::{bitcoin, Client, Error, RpcApi};

fn main_result() -> Result<(), Error> {
    let mut args = std::env::args();

    let _exe_name = args.next().unwrap();

    let server_pubkey = args.next().expect("Usage: <server_pubkey> [relay_url]");
    let relay_url = args.next().unwrap_or_else(|| "ws://localhost:10547".to_string());

    let rpc = Client::new(vec![relay_url], server_pubkey).unwrap();

    // Diagnostic: dump the server's advertised tools and their input schemas.
    rpc.dump_tool_schemas()?;

    let _blockchain_info = rpc.get_blockchain_info()?;

    let bestblockcount = rpc.get_block_count()?;
    println!("best block height: {}", bestblockcount);
    let best_block_hash = rpc.get_block_hash(bestblockcount)?;
    println!("best block hash by height: {}", best_block_hash);

    let bitcoin_block: bitcoin::Block = rpc.get_by_id(&best_block_hash)?;
    println!("best block hash by `get`: {}", bitcoin_block.header.prev_blockhash);
    let bitcoin_tx: bitcoin::Transaction =
        rpc.get_by_id(&bitcoin_block.txdata[0].compute_txid())?;
    println!("tx by `get`: {}", bitcoin_tx.compute_txid());

    Ok(())
}

fn main() {
    main_result().unwrap();
}
