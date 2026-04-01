use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct Registry<T> {
    items: HashMap<String, Arc<T>>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, name: &str, item: T) {
        self.items.insert(name.to_string(), Arc::new(item));
    }
    
    pub fn unregister(&mut self, name: &str) {
        self.items.remove(name);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<T>> {
        self.items.get(name).cloned()
    }
    
    pub fn list(&self) -> Vec<&str> {
        self.items.keys().map(|s| s.as_str()).collect()
    }
    
    pub fn count(&self) -> usize {
        self.items.len()
    }
}

pub type SourceRegistry = Registry<Box<dyn crate::traits::SourceFactory>>;
pub type EncoderRegistry = Registry<Box<dyn crate::traits::EncoderFactory>>;
pub type OutputRegistry = Registry<Box<dyn crate::traits::OutputFactory>>;