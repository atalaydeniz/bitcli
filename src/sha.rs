// For some reason, I've decided to implement SHA1 by hand instead of using a library. Might be a mistake.

use std::fs;

pub fn read_torrent(path: &str) -> std::io::Result<Vec<u8>> {
    return fs::read(path);
}