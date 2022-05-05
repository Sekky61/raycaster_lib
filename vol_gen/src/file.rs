/*
    vol_gen
    Author: Michal Majer
    Date: 2022-05-05
*/

use std::{
    fs::{File, OpenOptions},
    path::Path,
};

/// Open file
/// Existing files are rewritten
/// Non-existing files are created
pub fn open_create_file<P>(path: P) -> Result<File, std::io::Error>
where
    P: AsRef<Path>,
{
    let file = OpenOptions::new().write(true).create(true).open(path);

    file
}
