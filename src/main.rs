use futures::executor::block_on;
use octocrab::models::Code;
use octocrab::{models, Octocrab};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Serialize, Deserialize, Debug, Default)]
struct ModList {
    mods: HashMap<String, Mod>,
}

async fn get_pages(pat: String) -> Vec<Code> {
    let octocrab = Octocrab::builder()
        .personal_token(pat)
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

async fn populate_modlist(pages: Vec<Code>) -> ModList {
    let mut modlist = ModList::default();
    let mut stream = tokio_stream::iter(pages);
    while let Some(code) = stream.next().await {
        let repo_path: String = code
            .repository
            .full_name
            .unwrap_or_else(|| String::from("repo/undefined"));
        modlist
            .mods
            .entry(repo_path.clone())
            .and_modify(|modEntry| modEntry.is_modpack = true)
            .or_insert(Mod {
                name: code.name,
                repo: repo_path,
                is_modpack: false,
            });
    }

    modlist.mods.remove(&String::from("repo/undefined"));
    modlist
}

async fn get_modlist(pat: String) -> ModList {
    let pages = get_pages(pat).await;
    populate_modlist(pages).await
}

fn get_github_token() -> String {
    match std::env::var("GITHUBPAT") {
        Ok(pat) => pat,
        Err(_) => {
            panic!("Error reading Github personal Access token (Did you set it to GITHUBPAT?)")
        }
    }
}

fn write_mods(mods: ModList) {
    let file = File::create("mods.json").expect("couldn't access filesystem");
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &mods).expect("writing failed");
    writer.flush().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pat = get_github_token();
    let mods = get_modlist(pat).await;
    write_mods(mods);
    Ok(())
}
