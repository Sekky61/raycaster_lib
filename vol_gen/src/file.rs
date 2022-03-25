use std::{
    fs::{File, OpenOptions},
    path::Path,
};

pub fn open_create_file<P>(path: P) -> Result<File, std::io::Error>
where
    P: AsRef<Path>,
{
    let file = OpenOptions::new().write(true).create(true).open(path);

    file
}
