use std::env;
use std::io;

use rust_crypto::{get_file_names, sha_sum};

use rust_crypto::sha::SHA256;

extern crate hex;


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    for p in get_file_names(args)? {
        let hash = sha_sum(SHA256, &p).unwrap();
        println!("{} {}", hex::encode(hash), p.display());
    }

    Ok(())
}
