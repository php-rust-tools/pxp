use std::{
    collections::HashMap,
    path::PathBuf,
};

use clap::Parser;
use colored::Colorize;
use pxp_index::storage::{IndexStorageConfig, IndexStorageManager};
use pxp_index::Index as Indexer;
use rustyline::{error::ReadlineError, CompletionType, Config, DefaultEditor};

use crate::utils::pxp_home_dir;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Searches for indexed functions and classes for a given project.",
    after_help = "This command is only intended for use during development and testing."
)]
pub struct Search {
    #[clap(help = "The path to an indexed directory.")]
    path: PathBuf,

    #[clap(short, long, help = "Do not show progress bar.")]
    no_progress: bool,
}

pub fn search(args: Search) -> anyhow::Result<()> {
    if !args.path.exists() {
        anyhow::bail!("The path `{}` does not exist.", args.path.display());
    }

    let index = get_index_storage(args.path).load();

    repl(&index)?;

    Ok(())
}

fn get_index_storage(path: PathBuf) -> IndexStorageManager {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hash = hasher.finish();

    let index_file = pxp_home_dir()
        .unwrap()
        .join(format!("{hash}.json"))
        .to_str()
        .unwrap()
        .to_string();

    let mut driver_config: HashMap<String, String> = HashMap::new();
    driver_config.insert("path".to_string(), index_file);

    let config = IndexStorageConfig::new("file", driver_config);

    IndexStorageManager::new(config)
}

fn repl(index: &Indexer) -> anyhow::Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .auto_add_history(true)
        .build();

    let mut rl = DefaultEditor::with_config(config)?;

    let history = pxp_home_dir()?.join(".index_history");
    if !history.exists() {
        std::fs::write(&history, "")?;
    }

    rl.load_history(&history)?;

    loop {
        let readline = rl.readline("index> ");

        match readline {
            Ok(command) => handle(&command, index)?,
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(_) => break,
        }
    }

    rl.append_history(&history)?;

    Ok(())
}

fn handle(command: &str, index: &Indexer) -> anyhow::Result<()> {
    index.search_files(command).iter().for_each(|path| {
        println!("{}", path.display().to_string().green());
    });

    index.search_classes(command).iter().for_each(|function| {
        println!("{}", function.name().to_string().blue());
    });

    index.search_functions(command).iter().for_each(|class| {
        println!("{}", class.get_name().to_string().yellow());
    });

    Ok(())
}
