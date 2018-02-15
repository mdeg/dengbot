extern crate serde;
extern crate serde_json;

use deng::Deng;
use std::fs::File;
use std::io::BufReader;

pub fn store_deng(path: &str, deng: &[Deng]) -> Result<(), serde_json::Error> {
    let f = ::std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect(&format!("Could not open storage at {}", path));

    serde_json::to_writer(f, &deng)
}

pub fn read(path: &str) -> Vec<Deng> {
    match File::open(path) {
        Ok(f) => serde_json::from_reader(BufReader::new(&f)).expect("Could not deserialize dengs"),
        Err(_) => {
            File::create(path).expect("Could not create deng storage file");
            vec!()
        }
    }
}