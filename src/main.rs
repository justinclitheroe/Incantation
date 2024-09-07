use octocrab::models::Code;
use octocrab::{models, Octocrab};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Error, Write};
use std::string::String;
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
struct ModList {
    mods: HashMap<String, Mod>,
}

impl ModList {
    pub fn new() -> Self {
        Self {
            mods: HashMap::new(),
        }
    }

    fn get_github_token(&self) -> String {
        /// Pulls in a github token from GITHUBPAT
        std::env::var("GITHUBPAT").unwrap_or_else(|_| {
            panic!("Error reading Github personal Access token (Did you set it to GITHUBPAT?)")
        })
    }

    async fn get_pages(&self) -> Vec<Code> {
        /// Helper function for ModList Initialization,
        /// pulls in all files with the STEAMODDED header on GitHub
        let octocrab = Octocrab::builder()
            .personal_token(self.get_github_token())
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

    pub fn write_mods(&self) {
        /// Writes a Modlist to a json file
        let file = File::create("mods.json").expect("couldn't access filesystem");
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self).expect("writing failed");
        writer.flush().unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut mod_list = ModList::new();
    &mod_list.populate_modlist().await;
    mod_list.write_mods();
    Ok(())
}
