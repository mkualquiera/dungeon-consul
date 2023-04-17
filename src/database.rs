use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use serde::{de::DeserializeOwned, Serialize};

pub struct LockGuard<'a, T: Serialize + DeserializeOwned + Default> {
    guard: MutexGuard<'a, T>,
    db: &'a Database<T>,
}

impl<'a, T: Serialize + DeserializeOwned + Default> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        // Save the database to the file using serde yaml.
        let file = std::fs::File::create(&self.db.path);
        if let Ok(file) = file {
            let data: Result<(), serde_yaml::Error> = serde_yaml::to_writer(file, &*self.guard);
            if let Err(err) = data {
                eprintln!("Error saving database: {}", err);
            }
        } else {
            eprintln!("Error opening database file: {}", file.err().unwrap());
        }
    }
}

impl<'a, T: Serialize + DeserializeOwned + Default> std::ops::Deref for LockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T: Serialize + DeserializeOwned + Default> std::ops::DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

/// A database that can be accessed by multiple threads and is automatically saved to
/// a file.
pub struct Database<T: Default + Serialize + DeserializeOwned> {
    /// The data that is stored in the database.
    pub data: Arc<Mutex<T>>,
    path: PathBuf,
}

impl<T: Default + Serialize + DeserializeOwned> Database<T> {
    /// Creates a new database.
    pub fn new(path: PathBuf) -> Self {
        // Try to load the database from the file using serde yaml.
        let data = {
            let file = std::fs::File::open(&path);
            if let Ok(file) = file {
                let data: Result<T, serde_yaml::Error> = serde_yaml::from_reader(file);
                if let Ok(data) = data {
                    return Self {
                        data: Arc::new(Mutex::new(data)),
                        path,
                    };
                }
            }
            T::default()
        };

        Self {
            data: Arc::new(Mutex::new(data)),
            path,
        }
    }

    /// Get a lock
    pub fn lock(&self) -> LockGuard<T> {
        LockGuard {
            guard: self.data.lock().unwrap(),
            db: self,
        }
    }
}
