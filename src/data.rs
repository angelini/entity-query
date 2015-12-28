extern crate rustc_serialize;
extern crate bincode;

use csv;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, BufReader};
use bincode::SizeLimit;
use bincode::rustc_serialize as serialize;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct Datum {
    pub e: u32,
    pub a: String,
    pub v: String,
    pub t: u32,
}

impl Datum {
    #[allow(dead_code)]
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
}

#[derive(Debug)]
pub struct DBView<'a> {
    datums: Vec<&'a Datum>,
}

#[derive(Debug)]
pub enum LoadError {
    FileMissing(String),
    InvalidInput(String),
}

impl DB {
    #[allow(dead_code)]
    pub fn new(datums: Vec<Datum>) -> DB {
        DB { datums: datums }
    }

    #[allow(dead_code)]
    pub fn from_vec(datums: Vec<(u32, &str, &str, u32)>) -> DB {
        DB {
            datums: datums.iter()
                          .map(|t| Datum::new(t.0, t.1.to_string(), t.2.to_string(), t.3))
                          .collect::<Vec<Datum>>(),
        }
    }

    pub fn from_file<'a>(filename: &'a str) -> Result<DB, LoadError> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(LoadError::FileMissing(filename.to_string())),
        };
        let mut reader = BufReader::new(file);
        let decoded: Result<DB, serialize::DecodingError> =
            serialize::decode_from(&mut reader, SizeLimit::Infinite);

        match decoded {
            Ok(db) => Ok(db),
            Err(e) => Err(LoadError::InvalidInput(format!("file: {}\nerr: {}", filename, e))),
        }
    }

    pub fn from_csv<'a>(entity: &str, filename: &str, time: &str) -> Result<DB, LoadError> {
        let mut rdr = match csv::Reader::from_file(filename) {
            Ok(rdr) => rdr,
            Err(e) => {
                return Err(LoadError::InvalidInput(format!("file: {}\nerr: {}", filename, e)))
            }
        };
        let headers = rdr.headers().expect("Headers required to convert CSV");

        let time_index = match headers.iter().enumerate().find(|&(_, h)| h == time) {
            Some((idx, _)) => idx,
            None => return Err(LoadError::InvalidInput(format!("time header not found: {}", time))),
        };

        let mut eid = 0;
        let datums = rdr.records()
                        .flat_map(|row_res| {
                            let row = row_res.unwrap();
                            let time_val = row[time_index].parse::<u32>().unwrap();

                            eid += 1;
                            headers.iter()
                                   .zip(row)
                                   .map(|(header, val)| {
                                       Datum::new(eid,
                                                  format!("{}/{}", entity, robotize(header)),
                                                  val,
                                                  time_val)
                                   })
                                   .collect::<Vec<Datum>>()
                        })
                        .collect::<Vec<Datum>>();

        Ok(DB::new(datums))
    }

    pub fn filter<'a, F>(&'a self, predicate: F) -> DBView<'a>
        where F: Fn(&Datum) -> bool
    {
        DBView { datums: self.datums.iter().filter(|d| predicate(d)).collect::<Vec<&Datum>>() }
    }

    pub fn write(&self, filename: &str) -> serialize::EncodingResult<()> {
        let mut file = BufWriter::new(File::create(filename).unwrap());
        bincode::rustc_serialize::encode_into(self, &mut file, SizeLimit::Infinite)
    }
}

impl fmt::Display for DB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_datums(&self.datums.iter().collect(), f)
    }
}

impl<'a> fmt::Display for DBView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_datums(&self.datums, f)
    }
}

fn write_datums(datums: &Vec<&Datum>, f: &mut fmt::Formatter) -> fmt::Result {
    if datums.len() == 0 {
        try!(write!(f, "[]"))
    }

    for (idx, datum) in datums.iter().enumerate().take(5) {
        try!(write!(f, "{}", datum));
        if idx < datums.len() - 1 {
            try!(write!(f, "\n"));
        }
        if idx == 4 {
            try!(write!(f, "..."));
        }
    }
    Ok(())
}

fn robotize(string: &str) -> String {
    string.replace(" ", "_")
          .to_lowercase()
}
