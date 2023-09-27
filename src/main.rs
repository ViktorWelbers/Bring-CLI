use std::fs::create_dir_all;
use reqwest::Client;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use bring::BringClient;
use kv::Database;


mod bring;
mod kv;

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
    /// edit/show the shopping list uuid if you want to use a different list
    EditListUuid {},
    /// edit/show the auth token if you want to update it
    EditAuthtoken {},
    /// Show the shopping list
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


fn set_auth_token(database: &mut Database) -> String {
    let bearer = String::from("Bearer ");
    let mut auth_token = String::new();
    println!("Please enter your auth token:");
    std::io::stdin().read_line(&mut auth_token).unwrap();
    database.insert("auth_token".to_string(), auth_token.trim().to_string());
    bearer + &auth_token
}

fn set_list_uuid(database: &mut Database) -> String {
    let mut uuid = String::new();
    println!("Please enter the uuid of the list you want to use:");
    std::io::stdin().read_line(&mut uuid).unwrap();
    database.insert("list_uuid".to_string(), uuid.trim().to_string());
    uuid
}


#[tokio::main]
async fn main() {
    let mut path = PathBuf::from(r"C:\ProgramData\Bring");
    create_dir_all(path.clone()).expect("Could not create directory");
    path.push("kv.db");
    let mut database = Database::new(&path).expect("Database could not be created");


    let client = &Client::new();
    let cli = Cli::parse();
    let mut auth_token;
    let uuid;


    match database.get(&"auth_token".to_string()) {
        None => {
            auth_token = set_auth_token(&mut database);
        }
        Some(token) => {
            let bearer = String::from("Bearer ");
            auth_token = token.to_string();
            auth_token = bearer + &auth_token;
        }
    }

    match database.get(&"list_uuid".to_string()) {
        None => {
            uuid = set_list_uuid(&mut database);
        }
        Some(list_uuid) => {
            uuid = list_uuid.to_string();
        }
    }

    let bring_client = BringClient::new(&uuid, &auth_token);

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match cli.command {
        Some(Commands::List {}) => {
            bring_client.get_shopping_list(client)
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
        Some(Commands::Recipe(recipe)) =>
            {
                match recipe.command {
                    Some(RecipeCommands::Delete { recipe }) => {
                        database.remove(&recipe);
                        println!("Recipe {} deleted", recipe)
                    }
                    Some(RecipeCommands::Store { recipe }) => {
                        let mut name = recipe;
                        let mut ingredients = String::new();

                        println!("Enter the recipe ingredients for {} (separated by space):", name);
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
                                item
                                    .split(" ")
                                    .map(|s| s.to_string().remove(0).to_uppercase().to_string() + &s[1..])
                                    .collect::<Vec<String>>()
                            }
                            None => {
                                println!("Recipe {} not found. You need to it first via the 'store' command", recipe);
                                return;
                            }
                        };
                        bring_client.add_to_shopping_list(client, &items).await;
                    }
                    Some(RecipeCommands::Remove { recipe }) => {
                        let items = match database.get(&recipe) {
                            Some(item) => {
                                println!("Removing ingredients for {} to Bring list", recipe);
                                item
                                    .split(" ")
                                    .map(|s| s.to_string().remove(0).to_uppercase().to_string() + &s[1..])
                                    .collect::<Vec<String>>()
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
                    None => {}
                }
            }
        Some(Commands::EditListUuid {}) => {
            println!("Your list uuid is: {}", uuid);
            println!("Do you want to change it? (y/n)");
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer).unwrap();
            if answer.trim() == "y" {
                set_list_uuid(&mut database);
            }
        }
        Some(Commands::EditAuthtoken {}) => {
            println!("Your auth token is: {}", auth_token.strip_prefix("Bearer ").unwrap());
            println!("Do you want to change it? (y/n)");
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer).unwrap();
            if answer.trim() == "y" {
                set_auth_token(&mut database);
            }
        }
        None => {}
    }
}
