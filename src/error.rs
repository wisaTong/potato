use nix;
use std::io;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    NixError(nix::Error),
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Error {
        Error::NixError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}
