use crate::bring;
use chrono::Local;
use std::error::Error;

pub trait Storage {
    fn insert(&mut self, key: String, value: String);
    fn get(&self, key: &str) -> Option<&String>;
}

pub struct AuthInfo {
    pub auth_token: String,
    pub list_uuid: String,
}

pub fn auth_token() -> &'static str {
    "auth_token"
}

pub fn list_uuid() -> &'static str {
    "list_uuid"
}

pub fn expiration_timestamp() -> &'static str {
    "expiration_timestamp"
}

pub async fn new_login(storage: &mut dyn Storage) -> Result<AuthInfo, Box<dyn Error>> {
    println!("Please login with your credentials");
    println!("Enter your Bring! Mail: ");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    println!("Enter your Password: ");
    let password = rpassword::read_password().unwrap();
    let login_info = bring::request_bring_credentials(&username.trim(), &password.trim()).await?;
    storage.insert(auth_token().to_owned(), login_info.auth_token.to_string());
    storage.insert(list_uuid().to_owned(), login_info.list_uuid.to_string());
    storage.insert(
        expiration_timestamp().to_owned(),
        login_info.expiration_timestamp.to_string(),
    );
    Ok(AuthInfo {
        auth_token: login_info.auth_token,
        list_uuid: login_info.list_uuid,
    })
}
pub async fn use_stored_login(storage: &mut dyn Storage) -> Result<AuthInfo, Box<dyn Error>> {
    let mut token = String::new();
    let mut uuid = String::new();

    if let Some(saved_token) = storage.get(auth_token()) {
        token = "Bearer ".to_string() + saved_token;
    }

    if let Some(saved_uuid) = storage.get(list_uuid()) {
        uuid = saved_uuid.to_string();
    }

    if token.is_empty() || uuid.is_empty() {
        return new_login(storage).await;
    }

    match storage.get(expiration_timestamp()) {
        Some(expiration_date) => {
            if expiration_date.parse::<i64>().unwrap() < Local::now().timestamp() {
                println!("Auth token expired");
                return new_login(storage).await;
            }
        }
        None => {
            println!("No token expiration date found. Requesting new token");
            return new_login(storage).await;
        }
    }

    Ok(AuthInfo {
        auth_token: token,
        list_uuid: uuid,
    })
}
