use std::path::PathBuf;
use std::process;

use bring::Action;
use bring::BringClient;
use clap::{Parser, Subcommand};

mod bring;
mod database;
mod users;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Login with your Bring! credentials
    Login {},
    /// edit/show the shopping list uuid if you want to use a different list
    List {},
    /// Add an one or more items to the shopping list, seperated by space
    Add {
        #[arg(value_name = "ITEM")]
        item: Vec<String>,

        #[arg(short, long, value_name = "INFO")]
        info: Option<Vec<String>>,
    },
    /// Remove an one or more items from the shopping list, seperated by space
    Remove {
        #[arg(value_name = "ITEM")]
        item: Vec<String>,

        #[arg(short, long, value_name = "INFO")]
        info: Option<Vec<String>>,
    },
}

#[tokio::main]
async fn main() {
    let mut path = PathBuf::from(r"C:\ProgramData\Bring");
    let mut auth_token: String = String::new();
    let mut uuid: String = String::new();
    let mut database = match database::create_database(&mut path) {
        Ok(database) => database,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    let cli = Cli::parse();

    if let Some(Commands::Login {}) = cli.command {
        match users::new_login(&mut database).await {
            Ok(_auth_info) => {
                process::exit(1)
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    } else {
        match users::use_stored_login(&mut database).await {
            Ok(auth_info) => {
                auth_token = auth_info.auth_token;
                uuid = auth_info.list_uuid;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    let bring_client = BringClient::new(&uuid, &auth_token);

    match cli.command {
        Some(Commands::List {}) => {
            if let Err(e) = bring_client.get_shopping_list().await {
                println!("Error: {}", e);
            }
        }
        Some(Commands::Add { item, info }) => {
            if let Err(e) =
                bring::add_or_remove_item_shopping_list(bring_client, item, info, Action::ADD).await
            {
                println!("Error: {}", e);
            }
        }
        Some(Commands::Remove { item, info }) => {
            if let Err(e) =
                bring::add_or_remove_item_shopping_list(bring_client, item, info, Action::REMOVE)
                    .await
            {
                println!("Error: {}", e);
            }
        }
        Some(Commands::Login {}) => {}
        None => {
            println!("Please provide a command");
        }
    }
}
