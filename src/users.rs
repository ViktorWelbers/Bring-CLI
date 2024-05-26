use crate::bring;
use chrono::Local;
use std::error::Error;

pub trait Storage {
    fn insert(&mut self, key: String, value: String);
    fn get(&self, key: &str) -> Option<&String>;
    fn remove(&mut self, key: &str);
}

pub struct AuthInfo {
    pub auth_token: String,
    pub list_uuid: String,
}

pub struct UserMgmt{
    storage: dyn Storage
}

pub const AUTH_TOKEN: &str = "auth_token";
pub const LIST_UUID: &str = "list_uuid";
pub const EXPIRATION_TIMESTAMP: &str = "expiration_timestamp";
pub const USERNAME: &str = "username";

pub fn new_login(storage: &mut dyn Storage) -> Result<AuthInfo, Box<dyn Error>> {
    let username = match storage.get(USERNAME) {
        None => {
            let mut temp = String::new();
            println!("Please login with your credentials");
            println!("Enter your Bring! Mail: ");
            std::io::stdin().read_line(&mut temp)?;
            temp
        }

        Some(user) => {
            println!("Found existing user: {}", user);
            user.to_string()
        }
    };

    println!("Enter your Password: ");
    let password = rpassword::read_password().unwrap();
    let login_info = bring::request_bring_credentials(username.trim(), password.trim())?;
    storage.insert(AUTH_TOKEN.to_owned(), login_info.auth_token.to_string());
    storage.insert(LIST_UUID.to_owned(), login_info.list_uuid.to_string());
    storage.insert(USERNAME.to_owned(), username.to_string());
    storage.insert(
        EXPIRATION_TIMESTAMP.to_owned(),
        login_info.expiration_timestamp.to_string(),
    );
    Ok(AuthInfo {
        auth_token: login_info.auth_token,
        list_uuid: login_info.list_uuid,
    })
}

pub fn use_stored_login(storage: &mut dyn Storage) -> Result<AuthInfo, Box<dyn Error>> {
    let mut token = String::new();
    let mut uuid = String::new();

    if let Some(saved_token) = storage.get(AUTH_TOKEN) {
        token = "Bearer ".to_string() + saved_token;
    }

    if let Some(saved_uuid) = storage.get(LIST_UUID) {
        uuid = saved_uuid.to_string();
    }

    if token.is_empty() || uuid.is_empty() {
        return new_login(storage);
    }

    match storage.get(EXPIRATION_TIMESTAMP) {
        Some(expiration_date) => {
            if expiration_date.parse::<i64>().unwrap() < Local::now().timestamp() {
                println!("Auth token expired");
                return new_login(storage);
            }
        }
        None => {
            println!("No token expiration date found. Requesting new token");
            return new_login(storage);
        }
    }

    Ok(AuthInfo {
        auth_token: token,
        list_uuid: uuid,
    })
}

pub fn logout(storage: &mut dyn Storage) {
    storage.remove(USERNAME);
    storage.remove(EXPIRATION_TIMESTAMP);
    storage.remove(LIST_UUID);
    storage.remove(AUTH_TOKEN);
}
