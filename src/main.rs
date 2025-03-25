use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let args = Cli::parse();
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/",
        args.owner, args.repo
    );

    let client = Client::new();

    let _x = fetch_git_contents(&client, &url, &args.output);
}

fn fetch_git_contents(client: &Client, url: &String, output: &PathBuf) -> reqwest::Result<()> {
    let response = client
        .get(url)
        .header("User-Agent", "rust-cli")
        .send()?
        .json::<Vec<GitHubItem>>()?;

    // Process each item in the response
    for item in response {
        println!("New ITEM -> {:?}", item);
        match item {
            GitHubItem::File {
                path, download_url, ..
            } => {
                if let Some(url) = download_url {
                    let file_path = output.join(path);
                    println!("Downloading: {}", file_path.display());
                    download_file(client, &url, &file_path)?;
                }
            }
            GitHubItem::Directory { path, .. } => {
                let dir_url = format!("{}/{}", url.trim_end_matches('/'), path);
                let dir_path = output.join(path);
                match create_directory(&dir_path) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {e}");
                    }
                }
                fetch_git_contents(client, &dir_url, output)?; // Recursive call for the directory
            }
        }
    }

    Ok(())
}

fn create_directory(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        create_dir_all(path)
    } else {
        Ok(())  // The directory already exists, no need to do anything
    }
}

fn download_file(client: &Client, url: &str, file_path: &Path) -> reqwest::Result<()> {
    let response = client.get(url).header("User-Agent", "rust-cli").send()?;

    match response.bytes() {
        Ok(bytes) => {
            let mut file = File::create(file_path);
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
