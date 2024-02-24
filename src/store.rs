use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

type Key = String;

#[derive(Debug)]
pub struct StoreItem {
    pub value: String,
    pub expiry: Option<u128>,
}

type StoreType = Arc<Mutex<HashMap<Key, StoreItem>>>;

impl StoreItem {
    pub fn new(value: &str, expiry: Option<u64>) -> anyhow::Result<Self> {
        let value = value.to_owned();
        let mut expiry_time = None;

        if let Some(expiry) = expiry {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?;

            let current_time = current_time.as_millis();

            expiry_time = Some(current_time + expiry as u128);
        }

        Ok(Self {
            value,
            expiry: expiry_time,
        })
    }

    pub fn has_expired(&self) -> bool {
        if self.expiry.is_none() {
            return false;
        }

        let expiry = self.expiry.unwrap();

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Unable to get the current time");

        let current_time = current_time.as_millis();

        current_time > expiry
    }
}

#[derive(Default)]
pub struct Store(StoreType);

impl Deref for Store {
    type Target = StoreType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Store {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Store {
    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn insert(&mut self, key: &str, value: &str, expiry: Option<u64>) -> anyhow::Result<()> {
        let mut m = self.0.lock().unwrap();
        let item = StoreItem::new(value, expiry)?;
        m.insert(key.to_owned(), item);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let m = self.0.lock().unwrap();

        let item = m.get(key)?;

        if item.has_expired() {
            return None;
        }

        Some(item.value.to_owned())
    }
}
