use std::env;
use std::path::PathBuf;
use std::process::exit;

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
    /// Logout
    Logout {},
    /// Login with your Bring! credentials
    Login {},
    /// edit/show the shopping list uuid if you want to use a different list
    List {},
    /// Add one or more items to the shopping list, seperated by space
    Add {
        #[arg(value_name = "ITEM")]
        item: Vec<String>,

        #[arg(short, long, value_name = "INFO")]
        info: Option<Vec<String>>,
    },
    /// Remove one or more items from the shopping list, seperated by space
    Remove {
        #[arg(value_name = "ITEM")]
        item: Vec<String>,

        #[arg(short, long, value_name = "INFO")]
        info: Option<Vec<String>>,
    },
}

fn main() {
    let mut path = if env::consts::OS == "windows" {
        PathBuf::from(r"C:\ProgramData\Bring")
    } else {
        PathBuf::from("/var/lib/bring")
    };
    let mut auth_token: String = String::new();
    let mut uuid: String = String::new();
    let mut database = match database::create_database(&mut path) {
        Ok(database) => database,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    let mut user_management = users::UserManagement::new(&mut database);
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Login {}) => {
            match user_management.new_login() {
                Ok(_auth_info) => {}
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
            drop(database);
            exit(0);
        }
        Some(Commands::Logout {}) => {
            user_management.logout();
            println!("You were logged out");
            drop(database);
            exit(0)
        }
        _ => {}
    }

    match user_management.use_stored_login() {
        Ok(auth_info) => {
            auth_token = auth_info.auth_token;
            uuid = auth_info.list_uuid;
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let bring_client = BringClient::new(&uuid, &auth_token);

    match cli.command {
        Some(Commands::List {}) => {
            if let Err(e) = bring_client.get_shopping_list() {
                println!("Error: {}", e);
            }
        }
        Some(Commands::Add { item, info }) => {
            if let Err(e) =
                bring::add_or_remove_item_shopping_list(bring_client, item, info, Action::ADD)
            {
                println!("Error: {}", e);
            }
        }
        Some(Commands::Remove { item, info }) => {
            if let Err(e) =
                bring::add_or_remove_item_shopping_list(bring_client, item, info, Action::REMOVE)
            {
                println!("Error: {}", e);
            }
        }
        Some(Commands::Login {}) => {}
        Some(Commands::Logout {}) => {}
        None => {
            println!("Please provide a command");
        }
    }
}
