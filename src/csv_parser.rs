use csv;
use scoped_threadpool::Pool;
use std::collections::HashMap;

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
            None => return Err(Error::MissingTimeHeader(self.time.to_owned())),
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
                let filter = Filter::new(&db, pool);
                self.find_refs_for_join(datums, column, query, filter)
            })
            .collect::<Vec<Ref>>()
    }

    fn find_refs_for_join(&self, datums: &[Datum], column: &str, query: &str, filter: Filter)
                          -> Vec<Ref> {
        let attribute = format!("{}/{}", self.entity, robotize(&column));
        let ast = AstNode::parse(&query).unwrap();

        let new_datums = datums.iter()
                               .filter(|d| d.a == attribute)
                               .cloned()
                               .collect::<Vec<Datum>>();
        let old_datums = filter.execute(&ast).datums;

        let index = index_by_value(old_datums);

        new_datums.iter()
                  .flat_map(|new| Self::generate_refs(new, &index))
                  .filter(|o| o.is_some())
                  .map(|o| o.unwrap())
                  .collect()
    }

    fn generate_refs(new: &Datum, index: &HashMap<&str, Vec<&Datum>>) -> Vec<Option<Ref>> {
        let new_entity = new.a.split('/').next().unwrap();

        match index.get(new.v.as_str()) {
            Some(old_matches) => {
                old_matches.iter()
                           .map(|old| {
                               let old_entity = old.a.split('/').next().unwrap();
                               Some(Ref::new(new.e,
                                             format!("{}/{}", new_entity, old_entity),
                                             old.e,
                                             new.t))
                           })
                           .collect()
            }
            None => vec![None],
        }
    }

    fn parse_row(row: Vec<String>, headers: &[String], time_index: usize, eid: usize, entity: &str)
                 -> Result<Vec<Datum>, Error> {
        let time = match row[time_index].parse::<usize>() {
            Ok(t) => t,
            Err(_) => return Err(Error::TimeColumnTypeError(row[time_index].to_owned())),
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

fn index_by_value(datums: Vec<&Datum>) -> HashMap<&str, Vec<&Datum>> {
    let mut index = HashMap::new();
    for datum in datums {
        if let Some(mut foo) = index.insert(datum.v.as_str(), vec![datum]) {
            foo.push(datum)
        }
    }
    index
}
