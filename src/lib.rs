mod block_splitter;
mod padder;
mod primitives;
mod sha256;
mod sha512;
pub mod sha;
mod sha_tests;

use std::env;
use std::path::{PathBuf, Path};
use std::fs::{File,read_dir};
use std::convert::TryFrom;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;

use padder::Padder;
use sha::ShaParams;


fn is_file_or_complain(path: &PathBuf) -> bool {
    let name_of_executable = env::args().next().unwrap();

    let ok = path.is_file();
    if !ok {
        eprintln!("{}: {} is not a file", name_of_executable, path.display());
    }

    ok
}

pub fn get_file_names(args: Vec<String>) -> io::Result<Vec<PathBuf>> {
    let files: Vec<PathBuf>;
    if args.len()>1 {
        //explicit list of files; it's OK to say if something isn't a file
        files = args[1..].iter()
            .map(|s| Path::new(s).to_path_buf())
            .filter(|p| is_file_or_complain(p))
            .collect::<Vec<_>>();
    } else {
        // no files, so read current dir, and silently ignore all that isn't a file
        files = read_dir(".")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect::<Vec<_>>();
    }

    Ok(files)
}

pub fn sha_sum<T: std::clone::Clone>(algo: ShaParams<T>, file: &PathBuf) -> io::Result<Vec<u8>> {
    let f = File::open(file)?;
    let mut reader = BufReader::new(f);
    let mut count: u128 = 0;
    let mut msg = [0u8;128];
    let mut hash = algo.h0;
    let padder = Padder::new(algo.block_size, algo.length_size);

    loop {
        let len = reader.read(&mut msg[..algo.block_size])?;
        count += u128::try_from(len).unwrap();

        if padder.is_full_block(len) {
            hash = (algo.sha_func)(hash, &msg[..algo.block_size]);
        } else {
            if len == 0 {
                hash = (algo.sha_func)(hash, &padder.single_pad(&[], count));
            } else {
                if padder.is_room(len) {
                    hash = (algo.sha_func)(hash, &padder.single_pad(&msg[..len], count));
                } else {
                    hash = (algo.sha_func)(hash, &padder.double_pad_1st_part(&msg[..len]));
                    hash = (algo.sha_func)(hash, &padder.double_pad_2nd_part(count));
                }
            }
            break;
        }
    }

    Ok((algo.convert_func)(hash.to_vec()))
}
