use std::path::PathBuf;

use iroh::{Endpoint, protocol::Router};
use iroh_blobs::{BlobsProtocol, store::mem::MemStore, ticket::BlobTicket};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = Endpoint::bind().await?;
    let store = MemStore::new();
    let blobs = BlobsProtocol::new(&store, None);

    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();

    match args_ref.as_slice() {
        ["send", filename] => {
            let filename: PathBuf = filename.parse()?;
            let abs_path = std::path::absolute(&filename)?;
            println!("Hashing file...");
            let tag = store.blobs().add_path(abs_path).await?;
            let node_id = endpoint.id();
            let ticket = BlobTicket::new(node_id.into(), tag.hash, tag.format);

            println!("File hashed. Fetch this file by running:");
            println!("cargo run -- receive {ticket} {}", filename.display());

            let router = Router::builder(endpoint)
                .accept(iroh_blobs::ALPN, blobs)
                .spawn();

            tokio::signal::ctrl_c().await?;

            println!("Shutting down.");
            router.shutdown().await?;
        }
        ["receive", ticket, filename] => {
            let filename: PathBuf = filename.parse()?;
            let abs_path = std::path::absolute(filename)?;
            let ticket: BlobTicket = ticket.parse()?;

            let downloader = store.downloader(&endpoint);

            println!("Starting download.");

            downloader
                .download(ticket.hash(), Some(ticket.addr().id))
                .await?;

            println!("Finish download.");
            println!("Copying to destination.");

            store.blobs().export(ticket.hash(), abs_path).await?;

            println!("Finished copying.");

            println!("Shutting down.");
            endpoint.close().await;
        }
        _ => {
            println!("Error...")
        }
    }

    Ok(())
}
