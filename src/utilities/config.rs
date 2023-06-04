use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

const FILENAME: &str = "config.dat";

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub markov_chain_learning: HashSet<i64>,
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let path = Path::new(FILENAME);

        if path.exists() {
            log::debug!("loading bot config from drive");
            Ok(rmp_serde::decode::from_read(BufReader::new(File::open(path)?)).unwrap())
        } else {
            log::debug!("creating default bot config");
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> io::Result<()> {
        log::debug!("saving bot config to drive");
        let file = File::options().write(true).truncate(true).create(true).open(FILENAME)?;
        rmp_serde::encode::write_named(&mut BufWriter::new(file), self).unwrap();

        Ok(())
    }
}
