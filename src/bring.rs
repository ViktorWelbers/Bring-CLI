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

pub struct LoginInfo {
    pub auth_token: String,
    pub list_uuid: String,
    pub expiration_timestamp: i64,
}

pub struct BringClient {
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

pub async fn login_with_credentials(
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

        if res.status() != StatusCode::OK {
            println!("Login failed with the provided credentials failed.");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Login failed with the provided credentials failed.",
            )));
        }
        let response_body = res.text().await?;
        let response: HashMap<String, Value> = serde_json::from_str(&response_body)?;

        println!("Login successful!");
        Ok(response)
    }

    pub async fn get_shopping_list(&self, client: &Client) -> Result<(), Box<dyn Error>> {
        let res = make_request(
            client,
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
            println!("Something went wrong with the request");
        }
        Ok(())
    }

    pub async fn add_to_shopping_list(&self, client: &Client, items: &Vec<String>) {
        for item in items {
            let body = format!("uuid={}&purchase={}", self.list_uuid, item);
            let response = make_request(
                client,
                &self.url,
                body,
                &RequestType::PUT,
                Some(&self.auth_token),
            )
            .await
            .expect(&*format!(
                "Something went wrong with the request for item {}",
                item
            ));
            match response.status() {
                StatusCode::OK | StatusCode::NO_CONTENT => {
                    println!("{} added to shopping list!", item)
                }
                _ => {
                    println!(
                        "Could not add to shopping list due to error: {}",
                        response.status()
                    )
                }
            }
        }
    }

    pub async fn remove_from_shopping_list(&self, client: &Client, items: &Vec<String>) {
        for item in items {
            let body = format!("uuid={}&remove={}", self.list_uuid, item);
            let response = make_request(
                client,
                &self.url,
                body,
                &RequestType::PUT,
                Some(&self.auth_token),
            )
            .await
            .expect(&*format!(
                "Something went wrong with the request for item {}",
                item
            ));
            match response.status() {
                StatusCode::OK | StatusCode::NO_CONTENT => {
                    println!("{} removed from shopping list!", item)
                }
                _ => {
                    println!(
                        "Could not remove from shopping list due to error: {} ",
                        response.status()
                    )
                }
            }
        }
    }
}

pub fn unpack_ingredients_from_str(item: &str) -> Vec<String> {
    item.split(",")
        .map(|s| s.to_string().remove(0).to_uppercase().to_string() + &s[1..])
        .collect::<Vec<String>>()
}
