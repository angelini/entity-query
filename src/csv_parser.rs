use csv;
use scoped_threadpool::Pool;

use ast::AstNode;
use cli::Join;
use data::{Datum, Db, Ref, Error};
use filter::Filter;

#[derive(Debug)]
pub struct CsvParser<'a> {
    filename: &'a str,
    entity: &'a str,
    time: &'a str,
    joins: &'a [Join],
}

impl<'a> CsvParser<'a> {
    pub fn new(filename: &'a str, entity: &'a str, time: &'a str, joins: &'a [Join]) -> CsvParser<'a> {
        CsvParser {
            filename: filename,
            entity: entity,
            time: time,
            joins: joins,
        }
    }

    pub fn parse(self, db: &Db, pool: &mut Pool) -> Result<(Vec<Datum>, Vec<Ref>, usize), Error> {
        let mut rdr = try!(csv::Reader::from_file(self.filename));
        let headers = rdr.headers().expect("headers required to convert CSV");

        let time_index = match headers.iter()
                                      .enumerate()
                                      .find(|&(_, h)| h == self.time) {
            Some((idx, _)) => idx,
            None => {
                return Err(Error::MissingTimeHeader(self.time.to_owned()))
            }
        };

        let mut eid = db.offset;
        let datums_res = rdr.records()
                            .map(|row_res| {
                                let row = try!(row_res);
                                eid += 1;
                                let datums = try!(Self::parse_row(row,
                                                                  &headers,
                                                                  time_index,
                                                                  eid,
                                                                  self.entity));
                                Ok(datums)
                            })
                            .collect::<Result<Vec<Vec<Datum>>, Error>>();

        let datums = match datums_res {
            Ok(d) => d.into_iter().flat_map(|v| v).collect::<Vec<Datum>>(),
            Err(e) => return Err(e),
        };

        let refs = self.find_refs(&datums, &db, pool);
        Ok((datums, refs, eid))
    }

    fn find_refs(&self, datums: &[Datum], db: &Db, pool: &mut Pool) -> Vec<Ref> {
        self.joins
            .iter()
            .flat_map(|join| {
                let (column, query) = (&join.0, &join.1);
                let attribute = format!("{}/{}", self.entity, robotize(&column));
                let ast = AstNode::parse(&query).unwrap();

                let mut new_datums = datums.iter()
                                       .filter(|d| d.a == attribute)
                                       .cloned()
                                       .collect::<Vec<Datum>>();
                let mut old_datums = Filter::new(&db, pool).execute(&ast).datums;

                let mut old_idx = 0;
                new_datums.sort_by(|l, r| l.v.cmp(&r.v));
                old_datums.sort_by(|l, r| l.v.cmp(&r.v));

                new_datums.iter()
                          .map(|new| {
                              let new_entity = new.a.split('/').next().unwrap();
                              let old_datums_left = old_datums[old_idx..].iter();

                              match old_datums_left.enumerate().find(|&(_, o)| o.v == new.v) {
                                  Some((i, old)) => {
                                      old_idx = i;
                                      let old_entity = old.a.split('/').next().unwrap();
                                      Some(Ref::new(new.e,
                                                    format!("{}/{}", new_entity, old_entity),
                                                    old.e,
                                                    new.t))
                                  }
                                  None => None,
                              }
                          })
                          .filter(|o| o.is_some())
                          .map(|o| o.unwrap())
                          .collect::<Vec<Ref>>()
            })
            .collect::<Vec<Ref>>()
    }

    fn parse_row(row: Vec<String>, headers: &[String], time_index: usize, eid: usize, entity: &str)
                 -> Result<Vec<Datum>, Error> {
        let time = match row[time_index].parse::<usize>() {
            Ok(t) => t,
            Err(_) => {
                return Err(Error::TimeColumnTypeError(row[time_index].to_owned()))
            }
        };
        let datums = headers.iter()
                            .enumerate()
                            .filter(|&(i, _)| i != time_index)
                            .map(|(_, h)| h)
                            .zip(row)
                            .map(|(header, val)| {
                                Datum::new(eid,
                                           format!("{}/{}", entity, robotize(header)),
                                           val,
                                           time)
                            })
                            .collect();
        Ok(datums)
    }
}

fn robotize(string: &str) -> String {
    string.replace(" ", "_")
          .to_lowercase()
}
