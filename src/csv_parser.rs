use csv;
use ast::ASTNode;
use cli::Join;
use data::{Datum, DB, Ref, LoadError};
use filter::Filter;

#[derive(Debug)]
pub struct CSVParser<'a> {
    filename: &'a str,
    entity: &'a str,
    time: &'a str,
    joins: &'a [Join],
}

impl<'a> CSVParser<'a> {
    pub fn new(filename: &'a str, entity: &'a str, time: &'a str, joins: &'a [Join]) -> CSVParser<'a> {
        CSVParser {
            filename: filename,
            entity: entity,
            time: time,
            joins: joins,
        }
    }

    pub fn parse(self, db: &DB) -> Result<(Vec<Datum>, Vec<Ref>, usize), LoadError> {
        let mut rdr = match csv::Reader::from_file(self.filename) {
            Ok(rdr) => rdr,
            Err(e) => {
                return Err(LoadError::InvalidInput(format!("file: {}, err: {}", self.filename, e)))
            }
        };
        let headers = rdr.headers().expect("headers required to convert CSV");

        let time_index = match headers.iter()
                                      .enumerate()
                                      .find(|&(_, h)| h == self.time) {
            Some((idx, _)) => idx,
            None => {
                return Err(LoadError::InvalidInput(format!("time header not found: {}", self.time)))
            }
        };

        let mut eid = db.offset;
        let datums_res = rdr.records()
                            .map(|row_res| {
                                let row = row_res.unwrap();
                                eid += 1;
                                let datums = try!(Self::parse_row(row,
                                                                  &headers,
                                                                  time_index,
                                                                  eid,
                                                                  self.entity));
                                Ok(datums)
                            })
                            .collect::<Result<Vec<Vec<Datum>>, LoadError>>();

        let datums = match datums_res {
            Ok(d) => d.into_iter().flat_map(|v| v).collect::<Vec<Datum>>(),
            Err(e) => return Err(e),
        };

        let refs = self.find_refs(&datums, db);
        Ok((datums, refs, eid))
    }

    // TODO: Sort both datasets first and do a streaming join
    fn find_refs(&self, datums: &[Datum], db: &DB) -> Vec<Ref> {
        self.joins
            .iter()
            .flat_map(|join| {
                let (column, query) = (&join.0, &join.1);
                let attribute = format!("{}/{}", self.entity, robotize(&column));
                let ast = ASTNode::parse(&query).unwrap();

                let new_datums = datums.iter()
                                       .filter(|d| d.a == attribute)
                                       .cloned()
                                       .collect::<Vec<Datum>>();
                let old_datums = Filter::new(db, &ast, 12).execute().datums;

                new_datums.iter()
                          .map(|new| {
                              let new_entity = new.a.split('/').next().unwrap();
                              match old_datums.iter().find(|o| o.v == new.v) {
                                  Some(old) => {
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
                 -> Result<Vec<Datum>, LoadError> {
        let time = match row[time_index].parse::<usize>() {
            Ok(t) => t,
            Err(_) => {
                return Err(LoadError::InvalidInput(format!("time col is not an int: {}",
                                                           row[time_index])))
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
