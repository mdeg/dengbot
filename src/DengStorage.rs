extern crate serde;
extern crate serde_json;

use deng::Deng;
use constants::DENG_STORAGE;
use std::fs::File;
use std::io::BufReader;

pub fn create_storage() -> Vec<Deng> {
    File::create(DENG_STORAGE).expect("Could not create deng storage file");
    vec!()
}

pub fn store_deng(deng: &[Deng]) -> Result<(), serde_json::Error> {

    let f = ::std::fs::OpenOptions::new()
        .write(true)
        .open(DENG_STORAGE)
        .expect(&format!("Could not open storage at {}", DENG_STORAGE));

    serde_json::to_writer(f, &deng)
}

pub fn read_dengs() -> Result<Vec<Deng>, ::std::io::Error> {
    let f = File::open(DENG_STORAGE)?;
    Ok(serde_json::from_reader(BufReader::new(&f)).expect("Could not deserialize dengs"))
}