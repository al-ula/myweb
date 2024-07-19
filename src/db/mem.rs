use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::Error;

pub struct Data<K, V>
where K: Eq + Hash + Clone, V: Clone
{
    data: Arc<RwLock<HashMap<K, V>>>,
    key: Option<Arc<RwLock<Vec<K>>>>
}

impl <K: Eq + Hash + Clone, V: Clone> Data<K, V> {
    pub fn new(ordered: bool) -> Self {
        let key = match ordered {
            true => Some(Arc::new(RwLock::new(Vec::new()))),
            false => None
        };
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            key,
        }
    }

    pub fn from(ordered: bool, data: HashMap<K, V>) -> Self {
        let key = match ordered {
            true => Some(Arc::new(RwLock::new(Vec::new()))),
            false => None
        };
        Self {
            data: Arc::new(RwLock::new(data)),
            key,
        }
    }

    pub async fn get(&self, k: &K) -> Result<Option<V>, Error> {
        let read = self.data.try_read().map_err(|e| e.to_string())?;
        Ok(read.get(k).cloned())
    }

    pub async fn index_get(&self, index: usize) -> Result<Option<V>, Error> {
        let key = self.key.as_ref().ok_or("Not an ordered data")?.try_read().map_err(|e| e.to_string())?;
        if index < key.len() {
            Ok(self.data.try_read().map_err(|e| e.to_string())?.get(&key[index]).cloned())
        } else {
            Ok(None)
        }
    }

    pub async fn insert(&self, k: K, v: V) -> Result<(), Error> {
        if let Some(key) = &self.key {
            key.try_write().map_err(|e| e.to_string())?.push(k.clone());
        }
        self.data.try_write().map_err(|e| e.to_string())?.insert(k, v);
        Ok(())
    }

    pub async fn delete(&self, k: &K) -> Result<(), Error> {
        if let Some(key) = &self.key {
            key.try_write().map_err(|e| e.to_string())?.retain(|x| x != k);
        }
        self.data.try_write().map_err(|e| e.to_string())?.remove(k);
        Ok(())
    }

    pub async fn len(&self) -> Result<usize, Error> {
        if let Some(key) = &self.key {
            return Ok(key.try_read().map_err(|e| e.to_string())?.len())
        }
        Ok(self.data.try_read().map_err(|e| e.to_string())?.len())
    }

}