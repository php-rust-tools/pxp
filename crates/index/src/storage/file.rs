use super::IndexStorage;

pub struct FileStorage {
    config: std::collections::HashMap<String, String>,
}

impl FileStorage {
    pub fn new(config: &std::collections::HashMap<String, String>) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl IndexStorage for FileStorage {
    fn store(&mut self, index: &crate::Index) {
        let path = self.config.get("path").unwrap();

        if let Some(json) = serde_json::to_string(index).ok() {
            std::fs::write(path, json).unwrap();
        }
    }

    fn load(&self) -> crate::Index {
        let path = self.config.get("path").unwrap();

        let json = std::fs::read_to_string(path).unwrap();

        let index: crate::Index = serde_json::from_str(&json).unwrap();

        index
    }
}
