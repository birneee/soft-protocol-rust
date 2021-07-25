use std::collections::HashMap;
use std::sync::{Condvar, RwLock, Arc, Mutex};
use std::time::Instant;
use std::thread::Thread;
use std::thread;

struct AugmentedValue<V> {
    timeout: Instant,
    value: V
}

struct TtlMap<K,V> {
    inner: Arc<Mutex<HashMap<K, AugmentedValue<V>>>>,
    convar: Arc<Condvar>,
    min_timeout: Arc<Instant>
}

impl<K: 'static + Send, V: 'static + Send> TtlMap<K,V> {

    pub fn new() {

        let convar = Arc::new(Condvar::new());
        let hashmap = Arc::new(Mutex::new(HashMap::<K,V>::new()));

        thread::spawn(|| {
            let a = convar;
            let map = hashmap;
            //convar.wait(hashmap.lock().unwrap());
        });
    }

}