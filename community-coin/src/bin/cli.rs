//! A command-line interface for the Community Coin blockchain.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Get the balance of an address
    Balance {
        #[arg(short, long)]
        address: String,
    },
    /// Send coins to another address
    Transfer {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: u64,
    },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Balance { address } => {
            println!("Getting balance for address: {}", address);
        }
        Commands::Transfer { from, to, amount } => {
            println!("Transferring {} from {} to {}", amount, from, to);
        }
    }
}
