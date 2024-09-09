use octocrab::models::Code;
use octocrab::{models, Octocrab};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Error, Write};
use std::string::String;
use log::{info, log_enabled, Level};
use tokio_stream::StreamExt;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Mod {
    name: String,
    repo: String,
    is_modpack: bool,
}

impl fmt::Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, [{}])", self.name, self.repo)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ModListRaw {
    mods: HashMap<String, Mod>,
}

impl ModListRaw{
    pub fn new() -> Self {
        Self {
            mods: HashMap::new(),
        }
    }

    pub async fn get_pages(&self){
        info!("Polling GitHub API");
        /// Helper function for ModList Initialization,
        /// pulls in all files with the STEAMODDED header on GitHub

        /// Get initial pull of pages, take out the total_count and divide by 100
        /// (Max page size from the GitHub api), Get the ceiling and then subtract 1 because we already have the first page
        /// it would be cool to batch out the rest asynchronously
        let request_client = reqwest::Client::new();
        let pat = get_github_token();
        let first_page = request_client
            .get("https://api.github.com/search/code?&per_page=100&q=%22---%20STEAMODDED%20HEADER%22")
            // .header("Accept", "application/vnd.github.text-match+json")
            .header("Authorization", format!("Bearer {}", pat))
            .send().await.expect("request failed, might have gotten rate limited");

        println!("{first_page:?}");
    }

    pub async fn populate_modlist(&mut self) {
       todo!();
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ModListOctocrab {
    mods: HashMap<String, Mod>,
}


impl ModListOctocrab {
    pub fn new() -> Self {
        Self {
            mods: HashMap::new(),
        }
    }


    async fn get_pages(&self) -> Vec<Code> {
        info!("Polling GitHub API");
        /// Helper function for ModList Initialization,
        /// pulls in all files with the STEAMODDED header on GitHub
        let octocrab = Octocrab::builder()
            .personal_token(get_github_token())
            .build()
            .expect("Failed to create Octocrab Client");
        let pages = octocrab
            .search()
            .code("\"--- STEAMODDED HEADER\"")
            .per_page(100)
            .send()
            .await
            .expect("Search Failed");
        octocrab
            .all_pages::<Code>(pages)
            .await
            .expect("search failed")
    }

    pub async fn populate_modlist(&mut self) {
        info!("populating_modlist");
        let mut stream = tokio_stream::iter(self.get_pages().await);
        while let Some(code) = stream.next().await {
            let repo_path: String = code
                .repository
                .full_name
                .unwrap_or_else(|| String::from("repo/undefined"));
            self.mods
                .entry(repo_path.clone())
                .and_modify(|modEntry| modEntry.is_modpack = true)
                .or_insert(Mod {
                    name: code.name,
                    repo: repo_path,
                    is_modpack: false,
                });
        }
        &self.mods.remove(&String::from("repo/undefined"));
    }
}

fn get_github_token() -> String {
    /// Pulls in a github token from GITHUBPAT
    std::env::var("GITHUBPAT").unwrap_or_else(|_| {
        panic!("Error reading Github personal Access token (Did you set it to GITHUBPAT?)")
    })
}

pub fn write_mods(modlist: ModListRaw) {
    /// Writes a Modlist to a json file
    info!("Writing to file...");
    let file = File::create("mods.json").expect("couldn't access filesystem");
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &modlist).expect("writing failed");
    writer.flush().unwrap();
    info!("File Written")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    log_enabled!(Level::Info);
    let mut mod_list = ModListRaw::new();
    mod_list.get_pages().await;
    Ok(())
}
