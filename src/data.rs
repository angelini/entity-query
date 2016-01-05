use csv;
use bincode;
use bincode::SizeLimit;
use bincode::rustc_serialize as serialize;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use std::fmt;
use std::path;
use std::fs::File;
use std::io;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct Datum {
    pub e: usize,
    pub a: String,
    pub v: String,
    pub t: usize,
}

impl Datum {
    pub fn new<S>(e: usize, a: S, v: S, t: usize) -> Datum
        where S: Into<String>
    {
        Datum {
            e: e,
            a: a.into(),
            v: v.into(),
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
pub struct Ref {
    pub e: usize,
    pub a: String,
    pub v: usize,
    pub t: usize,
}

impl Ref {
    pub fn new(e: usize, a: String, v: usize, t: usize) -> Ref {
        Ref {
            e: e,
            a: a,
            v: v,
            t: t,
        }
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.e, self.a, self.v, self.t)
    }
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct Db {
    pub datums: Vec<Datum>,
    pub refs: Vec<Ref>,
    pub offset: usize,
}

#[derive(Debug)]
pub struct DbView<'a> {
    pub datums: Vec<&'a Datum>,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Csv(csv::Error),
    Encoding(serialize::EncodingError),
    Decoding(serialize::DecodingError),
    TimeColumnTypeError(String),
    MissingTimeHeader(String),
}

impl Db {
    pub fn new() -> Db {
        Db {
            datums: vec![],
            refs: vec![],
            offset: 0,
        }
    }

    pub fn from_file(filename: &str) -> Result<Db, Error> {
        let file = try!(File::open(filename));
        let reader = io::BufReader::new(file);
        let mut decoder = ZlibDecoder::new(reader);
        let decoded = try!(serialize::decode_from(&mut decoder, SizeLimit::Infinite));

        Ok(decoded)
    }

    pub fn write(&self, filename: &str) -> Result<(), Error> {
        let path = path::Path::new(filename);
        if path.exists() {
            return Err(Error::Io(io::Error::new(io::ErrorKind::AlreadyExists, filename)));
        }

        let writer = io::BufWriter::new(File::create(path).unwrap());
        let mut encoder = ZlibEncoder::new(writer, Compression::Fast);

        try!(bincode::rustc_serialize::encode_into(self, &mut encoder, SizeLimit::Infinite));
        Ok(())
    }

    pub fn insert(&mut self, datums: Vec<Datum>, refs: Vec<Ref>, offset: usize) {
        self.datums.extend(datums);
        self.refs.extend(refs);
        self.offset += offset;
    }
}

impl fmt::Display for Db {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "datums:\n"));
        try!(display_datums(&self.datums.iter().collect::<Vec<&Datum>>(), f, 20));
        try!(write!(f, "\nrefs:\n"));
        display_datums(&self.refs.iter().collect::<Vec<&Ref>>(), f, 20)
    }
}

impl<'a> fmt::Display for DbView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_datums(&self.datums, f, 20)
    }
}

fn display_datums<T>(datums: &[&T], f: &mut fmt::Formatter, size: usize) -> fmt::Result
    where T: fmt::Display
{
    if datums.len() == 0 {
        try!(write!(f, "()"))
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

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serialize::EncodingError> for Error {
    fn from(err: serialize::EncodingError) -> Error {
        Error::Encoding(err)
    }
}

impl From<serialize::DecodingError> for Error {
    fn from(err: serialize::DecodingError) -> Error {
        Error::Decoding(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}
