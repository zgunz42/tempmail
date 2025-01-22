use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct EmailStorage {
    emails: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
}

impl EmailStorage {
    pub fn new() -> Self {
        Self {
            emails: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_email(&self, address: String, raw_email: Vec<u8>) {
        let mut map = self.emails.lock().unwrap();
        map.entry(address).or_insert_with(Vec::new).push(raw_email);
    }

    pub fn get_emails(&self, address: &str) -> Option<Vec<Vec<u8>>> {
        let map = self.emails.lock().unwrap();
        map.get(address).cloned()
    }
}