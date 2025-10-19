use clap::{Parser, Subcommand};
use reqwest;
use serde_json::Value;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet management
    Wallet {
        #[clap(subcommand)]
        wallet_command: WalletCommands,
    },
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Get wallet balance
    Balance {
        #[clap(value_parser)]
        address: String,
    },
    /// Transfer coins to another wallet
    Transfer {
        #[clap(long)]
        from: String,
        #[clap(long)]
        to: String,
        #[clap(long)]
        amount: u64,
        #[clap(long)]
        private_key: String,
    },
    /// Get transaction history for a wallet
    History {
        #[clap(value_parser)]
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    match &cli.command {
        Commands::Wallet { wallet_command } => match wallet_command {
            WalletCommands::Balance { address } => {
                let res = client
                    .get(&format!("http://localhost:8000/wallet/{}", address))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res).unwrap());
            }
            WalletCommands::Transfer { from, to, amount, private_key } => {
                let res = client
                    .post("http://localhost:8000/transfer")
                    .json(&serde_json::json!({
                        "from": from,
                        "to": to,
                        "amount": amount,
                        "private_key": private_key,
                    }))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res).unwrap());
            }
            WalletCommands::History { address } => {
                let res = client
                    .get(&format!("http://localhost:8000/history/{}", address))
                    .send()
                    .await?
                    .json::<Value>()
                    .await?;
                println!("{}", serde_json::to_string_pretty(&res).unwrap());
            }
        },
    }
    Ok(())
}
