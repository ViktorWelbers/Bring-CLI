use reqwest::{Client, Response, StatusCode};
use std::collections::HashMap;
use serde_json::Value;
use hyper::header::AUTHORIZATION;

enum RequestType {
    GET,
    PUT,
}


pub struct BringClient {
    list_uuid: String,
    auth_token: String,
    url: String,
}

async fn make_request(client: &Client, url: &str, body: String, auth_token: &str, request_type: &RequestType) -> Result<Response, reqwest::Error> {
    let res = match request_type {
        RequestType::GET => client.get(url),
        RequestType::PUT => client.put(url).body(body),
    };

    let res = res.header(AUTHORIZATION, auth_token).send().await?;
    Ok(res)
}

impl BringClient {
    pub fn new(list_uuid: &str, auth_token: &str) -> Self {
        BringClient {
            list_uuid: String::from(list_uuid),
            auth_token: String::from(auth_token),
            url: String::from("https://api.getbring.com/rest/v2/bringlists/") + list_uuid,
        }
    }

    pub async fn get_shopping_list(&self, client: &Client) -> Result<HashMap<String, Value>, reqwest::Error> {
        let res = make_request(client, &self.url, String::from(""), &self.auth_token, &RequestType::GET).await?;

        let status: StatusCode = res.status().clone();
        let body: String = res.text().await?;

        let response: HashMap<String, Value> = serde_json::from_str(&body).unwrap();
        if status == StatusCode::OK {
            if let Some(purchase) = response.get("purchase") {
                if let Some(purchase_array) = purchase.as_array() {
                    println!("-------------");
                    for item in purchase_array {
                        let name = item["name"].as_str().unwrap_or("");
                        let specification = item["specification"].as_str().unwrap_or("");
                        if !specification.is_empty() {
                            println!("Name: {}", name);
                            println!("Info: {}", specification)
                        } else {
                            println!("Name: {}", name);
                        }
                        println!("-------------");
                    }
                }
            }
        }
        Ok(response)
    }

    pub async fn add_to_shopping_list(&self, client: &Client, items: &Vec<String>) {
        for item in items {
            let body = format!("uuid={}&purchase={}", self.list_uuid, item);
            make_request(client, &self.url, body, &self.auth_token, &RequestType::PUT).await.expect(&*format!("Something went wrong with the request for item {}", item));
            println!("{} added to shopping list!", item)
        }
    }

    pub async fn remove_from_shopping_list(&self, client: &Client, items: &Vec<String>) {
        for item in items {
            let body = format!("uuid={}&remove={}", self.list_uuid, item);
            make_request(client, &self.url, body, &self.auth_token, &RequestType::PUT).await.expect(&*format!("Something went wrong with the request for item {}", item));
            println!("{} removed from shopping list!", item)
        }
    }
}