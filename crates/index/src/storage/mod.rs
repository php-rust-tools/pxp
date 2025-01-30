mod file;

use std::{collections::HashMap, path::PathBuf};

use crate::Index;

pub trait IndexStorage {
    fn store(&mut self, index: &Index);
    fn load(&self) -> Index;
}

pub struct IndexStorageConfig {
    driver: &'static str,
    config: HashMap<String, String>,
}

impl IndexStorageConfig {
    pub fn new(driver: &'static str, config: HashMap<String, String>) -> Self {
        IndexStorageConfig { driver, config }
    }
}

pub struct IndexStorageManager {
    config: IndexStorageConfig,
    driver: Box<dyn IndexStorage>,
}

impl IndexStorageManager {
    pub fn new(config: IndexStorageConfig) -> Self {
        let driver: Box<dyn IndexStorage> = match config.driver {
            "file" => Box::new(file::FileStorage::new(&config.config)),
            _ => panic!("Unknown driver: {}", config.driver),
        };

        IndexStorageManager { config, driver }
    }

    pub fn store(&mut self, index: &Index) {
        self.driver.store(index);
    }

    pub fn load(&self) -> Index {
        self.driver.load()
    }
}
