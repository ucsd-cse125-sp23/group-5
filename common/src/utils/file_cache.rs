use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::hash::Hash;
use std::io::{BufWriter, Read, Write};
use std::marker::PhantomData;
use bincode::{deserialize, serialize};
use serde::{Serialize, Deserialize};
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

pub trait Cache<K, V> {
    fn get(&mut self, key: &K) -> Option<&V>;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
}

#[derive(Debug)]
pub struct FileCache<K, V> {
    file_path: String,
    cache: HashMap<K, V>,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> FileCache<K, V>
    where
        K: Serialize + DeserializeOwned + Eq + Hash + Clone,
        V: Serialize + DeserializeOwned + Clone,
{
    pub fn new(file_path: &str) -> Self {
        let cache = FileCache::load_from_file(file_path).unwrap_or(HashMap::new());
        FileCache {
            file_path: file_path.to_string(),
            cache,
            _marker: PhantomData,
        }
    }


    fn load_from_file(file_path: &str) -> Result<HashMap<K, V>> {
        let mut file = match OpenOptions::new().read(true).open(file_path) {
            Ok(file) => file,
            _ => return Ok(HashMap::new())
        };

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        let map: HashMap<K, V> = deserialize(&contents[..])
            .with_context(|| "Failed to deserialize the cache")?;

        Ok(map)
    }

    fn save_to_file(&self) -> Result<()> {
        let data = serialize(&self.cache).context("Failed to serialize the cache")?;
        let file = File::create(&self.file_path).context("Failed to create the cache file")?;
        let mut writer = BufWriter::new(file);
        writer
            .write_all(&data)
            .context("Failed to write the cache file")?;
        writer.flush().context("Failed to flush the cache file")?;
        Ok(())
    }
}

impl<K, V> Cache<K, V> for FileCache<K, V>
    where
        K: Serialize + DeserializeOwned + Eq + Hash + Clone,
        V: Serialize + DeserializeOwned + Clone,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let result = self.cache.insert(key.clone(), value.clone());
        if self.save_to_file().is_err() {
            eprintln!("Error saving cache to file");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use serde::{Serialize, Deserialize};
    use std::fs::remove_file;
    use crate::utils::file_cache::{Cache, FileCache};

    #[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
    struct Key(String);

    #[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct Value(i32);

    #[test]
    fn test_insert_and_get() {
        let file_path = "test_cache.bin";
        let mut cache = FileCache::<Key, Value>::new(file_path);

        let key = Key("key".to_string());
        let value = Value(42);

        assert!(cache.get(&key).is_none());
        cache.insert(key.clone(), value.clone());

        let cached_value = cache.get(&key);
        assert!(cached_value.is_some());
        assert_eq!(cached_value.unwrap(), &value);

        // Cleanup
        let _ = remove_file(file_path);
    }

    #[test]
    fn test_persistence() {
        let file_path = "test_cache_persistence.bin";

        // First run - insert value
        {
            let mut cache = FileCache::<Key, Value>::new(file_path);
            let key = Key("key".to_string());
            let value = Value(42);

            assert!(cache.get(&key).is_none());
            cache.insert(key.clone(), value.clone());
        }

        // Second run - read value from cache
        {
            let mut cache = FileCache::<Key, Value>::new(file_path);
            let key = Key("key".to_string());
            let value = Value(42);

            let cached_value = cache.get(&key);
            assert!(cached_value.is_some());
            assert_eq!(cached_value.unwrap(), &value);
        }

        // Cleanup
        let _ = remove_file(file_path);
    }
}