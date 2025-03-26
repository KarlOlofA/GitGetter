use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio;
use async_recursion::async_recursion;
use std::error::Error;
use std::fs;
use serde_json::Value;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/",
        args.owner, args.repo
    );

    let client = Client::new();
    
    match get_token() {
        Ok(token) => {
            match fetch_git_contents(&token, &client, &url, &args.output).await {
                Ok(user) => println!("{:#?}", user),
                Err(e) => eprintln!("Error fetching GitHub user: {}", e),
            }
        }
        Err(e) => eprintln!("Error reading token: {}", e),
    }
}

fn get_token() -> Result<String, Box<dyn Error>> {
    let file_content = fs::read_to_string("config.json")?;
    
    let json: Value = serde_json::from_str(&file_content)?;
    
    json["token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Token not found in config.json".into())
}

#[async_recursion]
async fn fetch_git_contents(token: &str, client: &Client, url: &String, output: &PathBuf) -> reqwest::Result<()> {

    let mut headers = HeaderMap::new();

    match HeaderValue::from_str(&format!("token {}", token)) {
        Ok(val) => {headers.insert(AUTHORIZATION, val);}
        Err(e) => {
            println!("{}", e);
            return Ok(())
        },
    }
    headers.insert(USER_AGENT, HeaderValue::from_static("GitGetter"));

    let response = client
        .get(url)
        .headers(headers)
        .send().await?;

    let x = response.json::<Vec<GitHubItem>>().await?;

    for item in x {
        println!("New ITEM -> {:?}", item);
        match item {
            GitHubItem::File {
                path, download_url, ..
            } => {
                if let Some(url) = download_url {
                    let file_path = output.join(path);
                    println!("Downloading: {}", file_path.display());
                    download_file(client, &url, &file_path).await?;
                }
            }
            GitHubItem::Directory { path, .. } => {
                let dir_url = format!("{}/{}", url, path);
                let dir_path = output.join(path);
                match create_directory(&dir_path) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {e}");
                    }
                }
                fetch_git_contents(token, client, &dir_url, output).await;
            }
        }
    }

    Ok(())
}

fn create_directory(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        create_dir_all(path)
    } else {
        Ok(()) 
    }
}

async fn download_file(client: &Client, url: &str, file_path: &Path) -> reqwest::Result<()> {
    let response = client.get(url).header("User-Agent", "rust-cli").send().await?;

    match response.bytes().await {
        Ok(bytes) => {
            let file = File::create(file_path);
            match file {
                Ok(mut file) => {
                    file.write_all(&bytes);},
                Err(e) => {println!("{e}");}
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to read bytes from response: {}", e);
            Err(e)
        }
    }
}

#[derive(Parser)]
struct Cli {
    owner: String,
    repo: String,
    #[arg(short, long, default_value = "./GitGetter")]
    output: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum GitHubItem {
    #[serde(rename = "file")]
    File {
        name: String,
        path: String,
        #[serde(rename = "download_url")]
        download_url: Option<String>,
    },
    #[serde(rename = "dir")]
    Directory { name: String, path: String },
}
