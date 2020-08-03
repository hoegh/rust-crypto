use std::env;
use std::fs::{File,read_dir};
use std::path::{PathBuf, Path};
use std::io::BufReader;
use std::io::prelude::*;
use std::io;

use rust_crypto::sha::{sha, SHA512};
extern crate hex;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let files: Vec<PathBuf>;
    if args.len()>1 {
        files = args[1..].iter()
            .map(|s| Path::new(s).to_path_buf())
            .collect::<Vec<_>>();
    } else {
        files = read_dir(".")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect::<Vec<_>>();
    }

    for p in files {
        let f = File::open(&p)?;
        let reader = BufReader::new(f);
        let hash = sha(SHA512, reader.bytes().map(Result::unwrap));

        println!("{} {}", hex::encode(hash), p.display());
    }

    Ok(())
}
