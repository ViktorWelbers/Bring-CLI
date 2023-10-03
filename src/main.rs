use chrono::Local;
use reqwest::Client;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;

use bring::BringClient;
use clap::{Args, Parser, Subcommand};
use database::Database;

mod bring;
mod database;

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
    Login {},
    /// edit/show the shopping list uuid if you want to use a different list
    List {},
    /// Add an one or more items to the shopping list, seperated by space
    Add {
        #[arg(value_name = "ITEMS")]
        items: Vec<String>,
    },
    /// Remove an one or more items from the shopping list, seperated by space
    Remove {
        #[arg(value_name = "ITEMS")]
        items: Vec<String>,
    },
    /// Manage recipes
    Recipe(RecipeCommand),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct RecipeCommand {
    #[command(subcommand)]
    command: Option<RecipeCommands>,
}

#[derive(Debug, Subcommand)]
enum RecipeCommands {
    /// Store a recipe with ingredients
    Store {
        #[arg(value_name = "RECIPE")]
        recipe: String,
    },
    /// Add a recipe to the shopping list
    Add {
        #[arg(value_name = "RECIPE")]
        recipe: String,
    },
    /// Remove a recipe from the shopping list
    Remove {
        #[arg(value_name = "RECIPE")]
        recipe: String,
    },
    /// List all stored recipes
    List {},
    /// Delete a recipe from the stored recipes
    Delete {
        #[arg(value_name = "RECIPE")]
        recipe: String,
    },
}

enum UserData {
    AuthToken,
    ListUuid,
    ExpirationTimestamp,
}

struct BringAuthInfo {
    auth_token: String,
    list_uuid: String,
}


impl UserData {
    fn as_string(&self) -> String {
        match self {
            UserData::AuthToken => "auth_token".to_string(),
            UserData::ListUuid => "list_uuid".to_string(),
            UserData::ExpirationTimestamp => "expiration_timestamp".to_string(),
        }
    }
}

async fn login_prompt(database: &mut Database) -> Result<BringAuthInfo, Box<dyn Error>> {
    println!("Please login with your credentials");
    println!("Enter your Bring! Mail: ");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();
    println!("Enter your Password: ");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).unwrap();
    let login_info =
        bring::login_with_credentials(&mut username.trim(), &mut password.trim()).await?;

    database.insert(
        UserData::AuthToken.as_string(),
        login_info.auth_token.to_string(),
    );
    database.insert(
        UserData::ListUuid.as_string(),
        login_info.list_uuid.to_string(),
    );
    database.insert(
        UserData::ExpirationTimestamp.as_string(),
        login_info.expiration_timestamp.to_string(),
    );
    Ok(BringAuthInfo {
        auth_token: login_info.auth_token,
        list_uuid: login_info.list_uuid,
    })
}

async fn fetch_authentication_data(database: &mut Database) -> BringAuthInfo {
    let mut token = String::new();
    let mut list_uuid = String::new();

    if let Some(saved_token) = database.get(&UserData::AuthToken.as_string()) {
        token = "Bearer ".to_string() + saved_token;
    }

    if let Some(saved_uuid) = database.get(&UserData::ListUuid.as_string()) {
        list_uuid = saved_uuid.to_string();
    }

    if token.is_empty() || list_uuid.is_empty() {
        return login_prompt(database).await.unwrap();
    }

    match database.get(&UserData::ExpirationTimestamp.as_string()) {
        Some(expiration_date) => {
            if expiration_date.parse::<i64>().unwrap() < Local::now().timestamp() {
                println!("Auth token expired");
                return login_prompt(database).await.unwrap();
            }
        }
        None => {
            println!("No token expiration date found. Requesting new token");
            return login_prompt(database).await.unwrap();
        }
    }

    BringAuthInfo {
        auth_token: token,
        list_uuid,
    }
}

fn create_database(path: &mut PathBuf) -> Database {
    create_dir_all(path.clone()).expect("Could not create directory");
    path.push("kv.db");
    Database::new(&path).expect("Database could not be created")
}

#[tokio::main]
async fn main() {
    let mut path = PathBuf::from(r"C:\ProgramData\Bring");
    let mut database = create_database(&mut path);
    let mut auth_token: String = String::new();
    let mut uuid: String = String::new();

    let client = &Client::new();
    let cli = Cli::parse();

    if let Some(Commands::Login {}) = cli.command {
        let login_result = login_prompt(&mut database).await;
        if let Some(bring) = login_result.ok() {
            auth_token = bring.auth_token;
            uuid = bring.list_uuid;
        }
    } else {
        let bring_auth_info = fetch_authentication_data(&mut database).await;
        auth_token = bring_auth_info.auth_token;
        uuid = bring_auth_info.list_uuid;
    }

    let bring_client = BringClient::new(&uuid, &auth_token);

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match cli.command {
        Some(Commands::List {}) => {
            bring_client
                .get_shopping_list(client)
                .await
                .expect("List could not be fetched. Try update your auth token in config");
        }
        Some(Commands::Add { items }) => {
            if items.is_empty() {
                println!("Please provide at least one item to remove.");
            } else {
                bring_client.add_to_shopping_list(client, &items).await;
            }
        }
        Some(Commands::Remove { items }) => {
            if items.is_empty() {
                println!("Please provide at least one item to remove.");
            } else {
                bring_client.remove_from_shopping_list(client, &items).await;
            }
        }
        Some(Commands::Recipe(recipe)) => match recipe.command {
            Some(RecipeCommands::Delete { recipe }) => {
                database.remove(&recipe);
                println!("Recipe {} deleted", recipe)
            }
            Some(RecipeCommands::Store { recipe }) => {
                let mut name = recipe;
                let mut ingredients = String::new();

                println!(
                    "Enter the recipe ingredients for {} (separated by comma):",
                    name
                );
                std::io::stdin().read_line(&mut ingredients).unwrap();

                name = name.trim().to_string();
                ingredients = ingredients.trim().to_string();
                ingredients.retain(|c| !c.is_whitespace());
                database.insert(name.clone(), ingredients);
                println!("Recipe {} stored", name)
            }
            Some(RecipeCommands::Add { recipe }) => {
                let items = match database.get(&recipe) {
                    Some(item) => {
                        println!("Adding ingredients for {} to Bring list", recipe);
                        bring::unpack_ingredients_from_str(item)
                    }
                    None => {
                        println!(
                            "Recipe {} not found. You need to it first via the 'store' command",
                            recipe
                        );
                        return;
                    }
                };
                println!("{}", items.join(","));
                bring_client.add_to_shopping_list(client, &items).await;
            }
            Some(RecipeCommands::Remove { recipe }) => {
                let items = match database.get(&recipe) {
                    Some(item) => {
                        println!("Removing ingredients for {} to Bring list", recipe);
                        bring::unpack_ingredients_from_str(item)
                    }
                    None => {
                        println!("Recipe {} not found. ", recipe);
                        return;
                    }
                };
                bring_client.remove_from_shopping_list(client, &items).await;
            }
            Some(RecipeCommands::List {}) => {
                let data = database.list();
                if data.is_empty() {
                    println!("No recipes stored");
                } else {
                    println!("Stored recipes:");
                    for (key, value) in data {
                        println!("{}: {}", key, value);
                    }
                }
            }
            None => {
                println!("No recipe command was used");
            }
        },
        None => {
            println!("No command was used");
        }
        _ => println!("No command was used"),
    }
}
