use reqwest::Client;
use std::path::{PathBuf};
use tokio;
use std::error::Error;
use std::fs;
use zip::read;
use serde::{Deserialize, Serialize}; 
use reqwest::header::{USER_AGENT};

#[tokio::main]
async fn main() {

    let json_data = read_json();

    let data = json_data.unwrap();
    download_repos(data).await;
}


async fn download_repos(data: Repositories) {
    for repo in &data.repositories {
        download_github_repo_as_zip(&repo.name, &repo.repo, &repo.branch).await.unwrap();
    }
}

async fn download_github_repo_as_zip(
    name: &str,
    repo: &str,
    branch: &str,
) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "{}/archive/refs/heads/{}.zip",
        repo, branch
    );

    println!("Starting download: {:?}", name);
    let output_path: PathBuf = format!("./FetchedRepos/{}.zip", name).into();
    let extract_path: PathBuf = format!("./FetchedRepos/{}", name).into();

    let client = Client::new();
    let response = client
        .get(&url)
        .header(USER_AGENT, "reqwest")
        .send()
        .await?;

    let bytes = response.bytes().await?;

    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::write(&output_path, &bytes);
    println!("Repository downloaded and saved to: {:?}", output_path);
    extract_zip_file(&output_path, &extract_path).await.unwrap();
    fs::remove_file(&output_path)?;
    Ok(())
}

fn read_json() -> Result<Repositories, Box<dyn Error>>{

    let path = "./repos.json";
    let data = fs::read_to_string(&path)?;
    let obj: Repositories = serde_json::from_str(&data)?;

    for repository in &obj.repositories {
        println!("{} - {}", repository.repo, repository.branch);
    }
    Ok(obj)
}

async fn extract_zip_file(zip_path: &PathBuf, extract_to: &PathBuf) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = read::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = extract_to.join(file.sanitized_name());

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Repositories {
    repositories: Vec<Repository>,
}

#[derive(Serialize, Deserialize)]
struct Repository {
    name: String,
    repo: String,
    branch: String,
}