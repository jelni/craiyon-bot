use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use markov_chain::MarkovChain;

const FILENAME: &str = "markov_chain.dat";
const MARKOV_CHAIN_ORDER: usize = 3;

pub fn load() -> io::Result<MarkovChain> {
    let path = Path::new(FILENAME);

    if path.exists() {
        log::debug!("loading Markov chain from drive");
        Ok(rmp_serde::decode::from_read(BufReader::new(File::open(path)?)).unwrap())
    } else {
        log::debug!("creating a new Markov chain");
        Ok(MarkovChain::new(MARKOV_CHAIN_ORDER))
    }
}

pub fn save(markov_chain: &MarkovChain) -> io::Result<()> {
    log::debug!("saving Markov chain to drive");
    let file = File::options().write(true).truncate(true).create(true).open(FILENAME)?;
    rmp_serde::encode::write(&mut BufWriter::new(file), markov_chain).unwrap();

    Ok(())
}

pub fn train(markov_chain: &mut MarkovChain, mut text: String) {
    text.make_ascii_lowercase();

    let mut words = text
        .split_ascii_whitespace()
        .map(|word| word.trim_matches(|char: char| !char.is_alphabetic()))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    if words.iter().any(|word| word.chars().count() > 32) {
        return;
    }

    words.dedup();

    markov_chain.train(words.into_iter().map(Into::into));
}
