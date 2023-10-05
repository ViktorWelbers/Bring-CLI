use chrono::Local;
use hyper::header::AUTHORIZATION;
use reqwest::{Client, Response, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

enum RequestType {
    GET,
    PUT,
    POST,
}
pub enum Action {
    REMOVE,
    ADD,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::REMOVE => write!(f, "removed from"),
            Action::ADD => write!(f, "added to"),
        }
    }
}

pub struct LoginInfo {
    pub auth_token: String,
    pub list_uuid: String,
    pub expiration_timestamp: i64,
}

pub struct BringClient {
    client: Client,
    list_uuid: String,
    auth_token: String,
    url: String,
}

async fn make_request(
    client: &Client,
    url: &str,
    body: String,
    request_type: &RequestType,
    auth_token: Option<&String>,
) -> Result<Response, reqwest::Error> {
    let res = match request_type {
        RequestType::GET => client.get(url),
        RequestType::PUT => client.put(url).body(body),
        RequestType::POST => client.post(url).body(body),
    };
    let res = match auth_token {
        Some(token) => res.header(AUTHORIZATION, token),
        None => res,
    };
    let res = res.send().await?;
    Ok(res)
}

pub async fn request_bring_credentials(
    user: &str,
    password: &str,
) -> Result<LoginInfo, Box<dyn Error>> {
    let client = Client::new();
    let response = BringClient::get_token_and_list_uuid(&client, user, password).await?;
    let auth_token = response["access_token"]
        .as_str()
        .ok_or("Could not get token")?
        .to_string();
    let list_uuid = response["bringListUUID"]
        .as_str()
        .ok_or("Could not get uuid")?
        .to_string();
    let expires_in_seconds = response["expires_in"]
        .as_i64()
        .ok_or("Could not get expires")?;
    let expiration_timestamp = Local::now().timestamp() + expires_in_seconds;

    Ok(LoginInfo {
        auth_token,
        list_uuid,
        expiration_timestamp,
    })
}

impl BringClient {
    pub fn new(list_uuid: &str, auth_token: &str) -> Self {
        BringClient {
            client: Client::new(),
            url: String::from("https://api.getbring.com/rest/v2/bringlists/") + list_uuid,
            list_uuid: String::from(list_uuid),
            auth_token: String::from(auth_token),
        }
    }

    async fn get_token_and_list_uuid(
        client: &Client,
        email: &str,
        password: &str,
    ) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        let url = String::from("https://api.getbring.com/rest/v2/bringauth");
        let request_body = format!("email={}&password={}", email, password);
        let res = make_request(client, &url, request_body, &RequestType::POST, None).await?;
        let status_code = res.status();
        if status_code != StatusCode::OK {
            println!("Login failed with the provided credentials failed.");
            let response_body = res.text().await?;
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not login {}: {}", status_code, response_body),
            )));
        }
        let response_body = res.text().await?;
        let response: HashMap<String, Value> = serde_json::from_str(&response_body)?;

        println!("Login successful!");
        Ok(response)
    }

    pub async fn get_shopping_list(&self) -> Result<(), Box<dyn Error>> {
        let res = make_request(
            &self.client,
            &self.url,
            String::from(""),
            &RequestType::GET,
            Some(&self.auth_token),
        )
        .await?;

        let status: StatusCode = res.status();
        let body: String = res.text().await?;

        if status == StatusCode::OK {
            let response: HashMap<String, Value> = serde_json::from_str(&body)?;
            if let Some(purchase) = response.get("purchase") {
                if let Some(purchase_array) = purchase.as_array() {
                    println!("-------------");
                    for item in purchase_array {
                        let name = item["name"].as_str().unwrap_or("");
                        match item["specification"].as_str() {
                            Some(specification) => {
                                println!("Name: {}", name);
                                println!("Info: {}", specification)
                            }
                            None => {
                                println!("Name: {}", name);
                            }
                        }
                        println!("-------------");
                    }
                }
            }
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not get shopping list {}: {}", status, body),
            )));
        }
        Ok(())
    }

    /// edit_shopping_list changes the shopping list by adding or removing an item
    /// from the list.
    ///
    /// The item is specified by the item parameter.
    ///
    /// The specification parameter is optional and is used to specify the amount of the item.
    ///
    /// The action parameter is used to specify whether the item should be added or
    /// removed from the list.
    pub async fn edit_shopping_list(
        &self,
        mut item: String,
        specification: Option<&str>,
        action: Action,
    ) -> Result<(), Box<dyn Error>> {
        item = capitalize_first_letter(&item);
        let body = match action {
            Action::ADD => format!("uuid={}&purchase={}", self.list_uuid, item,),
            Action::REMOVE => format!("uuid={}&remove={}", self.list_uuid, item),
        };
        let body = match specification {
            Some(spec) => format!("{}&specification={}", body, spec),
            None => body,
        };
        let response = make_request(
            &self.client,
            &self.url,
            body,
            &RequestType::PUT,
            Some(&self.auth_token),
        )
        .await?;

        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => {
                println!("{} {} shopping list!", item, action)
            }
            _ => {
                let body = response.text().await?;
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not {} {} {}: {}", action, item, self.list_uuid, body),
                )));
            }
        }
        Ok(())
    }
}

pub fn capitalize_first_letter(item: &str) -> String {
    let mut chars = item.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str(),
    }
}

pub async fn add_or_remove_item_shopping_list(
    client: BringClient,
    item: Vec<String>,
    info: Option<Vec<String>>,
    action: Action,
) -> Result<(), Box<dyn Error>> {
    if item.is_empty() {
        println!("Please provide an item to remove.");
    } else {
        let item = item.join(" ");
        if let Some(info_str) = info {
            client
                .edit_shopping_list(item, Some(info_str.join(" ").as_str()), action)
                .await?;
        } else {
            client.edit_shopping_list(item, None, action).await?;
        }
    }
    Ok(())
}
