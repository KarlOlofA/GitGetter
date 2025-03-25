use clap::Parser;
use reqwest::blocking::Client;

fn main() {
    let owner = "tomjerbo";
    let repo = "DevConsole";
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/");

    let client = Client::new();

    let _x = fetch_git_contents(&client, &url);
}

fn fetch_git_contents(client: &Client, url: &String) -> reqwest::Result<()> {
    match client.get(url).header("User-Agent", "RustCli").send() {
        Ok(response) => {
            if response.status().is_success() {
                match response.text() {
                    Ok(body) => println!("Response: {}", body),
                    Err(e) => eprintln!("Failed to read response: {}", e),
                }
            } else {
                eprintln!("Request failed with status: {}", response.status());
            }
        }
        Err(e) => eprintln!("Request error: {}", e),
    }
    Ok(())
}

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
}
