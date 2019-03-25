use crate::errors::ZugzugError;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bucket {
    pub name: String,
    pub path: String,
}

impl Bucket {
    fn pathbuf(&self) -> &Path {
        Path::new(&self.path)
    }

    pub fn make_dir(&self, name: &str) -> Result<PathBuf, Box<dyn error::Error + 'static>> {
        let now: DateTime<Local> = Local::now();
        let full_name = format!(
            "{:04}{:02}{:02}_{}",
            now.year(),
            now.month(),
            now.day(),
            name
        );
        let path = self.pathbuf().join(full_name);
        if path.exists() {
            return Err(Box::new(ZugzugError::new("Path already exists")));
        }
        fs::create_dir(&path)?;
        Ok(path)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct StoreData {
    pub default_bucket: Option<String>,
    pub buckets: Vec<Bucket>,
}

pub struct Store {
    location: PathBuf,
    data: StoreData,
    bucket_names: HashSet<String>,
}

impl Store {
    pub fn add_bucket(
        &mut self,
        name: &str,
        dir: &str,
    ) -> Result<(), Box<dyn error::Error + 'static>> {
        self.data.buckets.push(Bucket {
            name: name.to_string(),
            path: dir.to_string(),
        });
        match self.default_bucket() {
            None => {
                self.set_default_bucket(name)?;
            }
            Some(_) => {}
        }
        self.persist()
    }

    pub fn buckets(&self) -> Vec<Bucket> {
        self.data.buckets.clone()
    }

    pub fn find_bucket(&self, name: &str) -> Option<&Bucket> {
        self.data.buckets.iter().filter(|b| b.name == name).next()
    }

    pub fn default_bucket(&self) -> Option<&Bucket> {
        self.data
            .default_bucket
            .as_ref()
            .and_then(|name| self.data.buckets.iter().find(|&b| b.name == *name))
    }

    pub fn set_default_bucket(
        &mut self,
        name: &str,
    ) -> Result<(), Box<dyn error::Error + 'static>> {
        match self.find_bucket(name) {
            Some(_) => {
                self.data.default_bucket = Some(name.to_string());
                self.persist()?;
                Ok(())
            }
            None => Err(Box::new(ZugzugError::new("Bucket doesn't exist"))),
        }
    }

    pub fn unset_default_bucket(&mut self) -> Result<(), Box<dyn error::Error + 'static>> {
        self.data.default_bucket = None;
        self.persist()
    }

    pub fn forget_bucket(&mut self, name: &str) -> Result<(), Box<dyn error::Error + 'static>> {
        match self.default_bucket() {
            Some(bucket) => {
                if bucket.name == name {
                    self.unset_default_bucket()?;
                }
            }
            None => {}
        }
        self.data.buckets = self
            .data
            .buckets
            .iter()
            .filter(|bucket| bucket.name != name)
            .map(|bucket| bucket.clone())
            .collect();
        self.persist()
    }

    fn new(location: PathBuf) -> Store {
        let buckets: Vec<Bucket> = vec![];
        Store {
            location: location,
            data: StoreData {
                buckets: buckets,
                default_bucket: None,
            },
            bucket_names: HashSet::new(),
        }
    }

    // load the store from the home directory
    pub fn from_home() -> Result<Store, Box<dyn error::Error + 'static>> {
        if let Some(location) = dirs::home_dir() {
            Ok(Store::new(location))
        } else {
            Err(Box::new(ZugzugError::new("Could not get home directory")))
        }
    }

    // Initialize Store data and persist to disk
    fn init(&self) -> Result<(), Box<dyn error::Error + 'static>> {
        let path = self.store_path();
        fs::write(path, serde_json::to_string(&self.data)?)?;
        Ok(())
    }

    // construct the Store's data file path
    fn store_path(&self) -> PathBuf {
        Path::new(&self.location).join(".zz.json")
    }

    // persist Store contents to disk
    fn persist(&self) -> Result<(), Box<dyn error::Error + 'static>> {
        fs::write(self.store_path(), serde_json::to_string(&self.data)?)?;
        Ok(())
    }

    // load Store contents from disk
    fn internal_load(&mut self) -> Result<(), Box<dyn error::Error + 'static>> {
        let data = String::from_utf8(fs::read(self.store_path())?)?;
        let v: StoreData = serde_json::from_str(&data)?;
        let mut names = HashSet::new();
        for bucket in v.buckets.iter() {
            names.insert(bucket.name.to_string());
        }
        self.data = v;
        self.bucket_names = names;
        Ok(())
    }

    pub fn load() -> Result<Store, Box<dyn error::Error + 'static>> {
        let mut store = Store::from_home()?;
        if !store.store_path().exists() {
            println!("location does not exist yet");
            store.init()?;
        }
        store.internal_load()?;
        Ok(store)
    }
}
