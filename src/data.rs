extern crate rustc_serialize;
extern crate bincode;

use std::fs::File;
use std::io::{BufWriter, BufReader};
use bincode::SizeLimit;
use bincode::rustc_serialize::EncodingResult;

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

#[derive(Debug, Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct DB {
    pub datums: Vec<Datum>,
}

#[derive(Debug)]
pub struct DBView<'a> {
    datums: Vec<&'a Datum>,
}

impl DB {
    pub fn new(datums: Vec<Datum>) -> DB {
        DB { datums: datums }
    }

    pub fn from_vec(datums: Vec<(u32, &str, &str, u32)>) -> DB {
        DB {
            datums: datums.iter()
                          .map(|t| Datum::new(t.0, t.1.to_string(), t.2.to_string(), t.3))
                          .collect::<Vec<Datum>>(),
        }
    }

    pub fn from_file(filename: &str) -> DB {
        let mut reader = BufReader::new(File::open(filename).unwrap());
        bincode::rustc_serialize::decode_from(&mut reader, SizeLimit::Infinite).unwrap()
    }

    pub fn filter<'a, F>(&'a self, predicate: F) -> DBView<'a>
        where F: Fn(&Datum) -> bool
    {
        DBView { datums: self.datums.iter().filter(|d| predicate(d)).collect::<Vec<&Datum>>() }
    }

    pub fn write(&self, filename: &str) -> EncodingResult<()> {
        let mut file = BufWriter::new(File::create(filename).unwrap());
        bincode::rustc_serialize::encode_into(self, &mut file, SizeLimit::Infinite)
    }
}
