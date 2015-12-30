use std::fmt;
use std::path;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use bincode;
use bincode::SizeLimit;
use bincode::rustc_serialize as serialize;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct Datum {
    pub e: u32,
    pub a: String,
    pub v: String,
    pub t: u32,
}

impl Datum {
    pub fn new(e: u32, a: String, v: String, t: u32) -> Datum {
        Datum {
            e: e,
            a: a,
            v: v,
            t: t,
        }
    }
}

impl fmt::Display for Datum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.e, self.a, self.v, self.t)
    }
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct DB {
    pub datums: Vec<Datum>,
    pub offset: u32,
}

#[derive(Debug)]
pub struct DBView<'a> {
    pub datums: Vec<&'a Datum>,
}

#[derive(Debug)]
pub enum LoadError {
    FileMissing(String),
    InvalidInput(String),
}

#[derive(Debug)]
pub enum WriteError {
    FileExists(String),
    EncodingError(String),
}

impl DB {
    pub fn new() -> DB {
        DB {
            datums: vec![],
            offset: 0,
        }
    }

    pub fn from_file(filename: &str) -> Result<DB, LoadError> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(LoadError::FileMissing(filename.to_owned())),
        };
        let reader = BufReader::new(file);
        let mut decoder = ZlibDecoder::new(reader);
        let decoded: Result<DB, serialize::DecodingError> =
            serialize::decode_from(&mut decoder, SizeLimit::Infinite);

        match decoded {
            Ok(db) => Ok(db),
            Err(e) => Err(LoadError::InvalidInput(format!("file: {}, err: {}", filename, e))),
        }
    }

    pub fn write(&self, filename: &str) -> Result<(), WriteError> {
        let path = path::Path::new(filename);
        if path.exists() {
            return Err(WriteError::FileExists(filename.to_owned()));
        }

        let writer = BufWriter::new(File::create(path).unwrap());
        let mut encoder = ZlibEncoder::new(writer, Compression::Fast);

        match bincode::rustc_serialize::encode_into(self, &mut encoder, SizeLimit::Infinite) {
            Ok(_) => Ok(()),
            Err(e) => Err(WriteError::EncodingError(format!("file: {}, err: {}", filename, e))),
        }
    }
}

impl fmt::Display for DB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_datums(&self.datums.iter().collect::<Vec<&Datum>>(), f, 20)
    }
}

impl<'a> fmt::Display for DBView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_datums(&self.datums, f, 20)
    }
}

fn display_datums(datums: &[&Datum], f: &mut fmt::Formatter, size: usize) -> fmt::Result {
    if datums.len() == 0 {
        try!(write!(f, "[]"))
    }

    for (idx, datum) in datums.iter().enumerate().take(size) {
        try!(write!(f, "{}", datum));
        if idx < datums.len() - 1 {
            try!(write!(f, "\n"));
        }
        if idx == size - 1 {
            try!(write!(f, "..."));
        }
    }
    Ok(())
}
