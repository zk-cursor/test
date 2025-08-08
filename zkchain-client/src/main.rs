use anyhow::Result;
use clap::{Parser, Subcommand};
use reqwest::Client;
use zk::tx::{generate_keypair, Address, Tx};
use zk::zkps::RangeProver;

#[derive(Parser)]
#[command(name = "zkchain-client")]
#[command(about = "A simple client to send zk transactions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GenKey,
    SendTx { to_hex: String, amount: u64, #[arg(default_value_t=32)] bits: usize },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::GenKey => gen_key().await?,
        Commands::SendTx { to_hex, amount, bits } => send_tx(to_hex, amount, bits).await?,
    }
    Ok(())
}

async fn gen_key() -> Result<()> {
    let (sk, vk, addr) = generate_keypair();
    println!("pub: {}", hex::encode(vk.to_bytes()));
    println!("addr: {}", hex::encode(addr.0));
    println!("sk: {}", base64::encode(sk.to_bytes()));
    Ok(())
}

async fn send_tx(to_hex: String, amount: u64, bits: usize) -> Result<()> {
    let (sk, _vk, _addr) = generate_keypair();
    let mut to_bytes = [0u8; 32];
    let bytes = hex::decode(to_hex)?;
    to_bytes.copy_from_slice(&bytes[..32]);
    let to = Address(to_bytes);
    let prover = RangeProver::default();
    let tx = Tx::new_signed(&sk, to, amount, bits, &prover)?;
    let client = Client::new();
    let resp = client.post("http://127.0.0.1:8080/submit_tx").json(&tx).send().await?;
    println!("{}", resp.text().await?);
    Ok(())
}

