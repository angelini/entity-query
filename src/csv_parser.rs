use csv;
use ast::ASTNode;
use cli::Join;
use data::{Datum, DB, Ref, LoadError};
use filter::Filter;

// TODO: Sort both datasets first and do a streaming join
pub fn find_refs(datums: &[Datum], db: &DB, entity: &str, join: Join) -> Vec<Ref> {
    let (column, query) = (join.0, join.1);
    let attribute = format!("{}/{}", entity, robotize(&column));
    let ast = ASTNode::parse(&query).unwrap();

    let new_datums = datums.iter().filter(|d| d.a == attribute).cloned().collect::<Vec<Datum>>();
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
}

pub fn parse(filename: &str, entity: &str, time: &str, offset: u32)
             -> Result<(u32, Vec<Datum>), LoadError> {
    let mut rdr = match csv::Reader::from_file(filename) {
        Ok(rdr) => rdr,
        Err(e) => return Err(LoadError::InvalidInput(format!("file: {}, err: {}", filename, e))),
    };
    let headers = rdr.headers().expect("headers required to convert CSV");

    let time_index = match headers.iter()
                                  .enumerate()
                                  .find(|&(_, h)| h == time) {
        Some((idx, _)) => idx,
        None => return Err(LoadError::InvalidInput(format!("time header not found: {}", time))),
    };

    let mut eid = offset;
    let datums = rdr.records()
                    .map(|row_res| {
                        let row = row_res.unwrap();
                        let (offset, datums) = try!(parse_row(row,
                                                              &headers,
                                                              time_index,
                                                              eid,
                                                              entity));
                        eid = offset;
                        Ok(datums)
                    })
                    .collect::<Result<Vec<Vec<Datum>>, LoadError>>();

    match datums {
        Ok(d) => Ok((eid, d.into_iter().flat_map(|v| v).collect())),
        Err(e) => Err(e),
    }
}

fn parse_row(row: Vec<String>, headers: &[String], time_index: usize, eid: u32, entity: &str)
             -> Result<(u32, Vec<Datum>), LoadError> {
    let time = match row[time_index].parse::<u32>() {
        Ok(t) => t,
        Err(_) => {
            return Err(LoadError::InvalidInput(format!("time col is not an int: {}",
                                                       row[time_index])))
        }
    };
    let mut eid = eid;
    let datums = headers.iter()
                        .enumerate()
                        .filter(|&(i, _)| i != time_index)
                        .map(|(_, h)| h)
                        .zip(row)
                        .map(|(header, val)| {
                            eid += 1;
                            Datum::new(eid, format!("{}/{}", entity, robotize(header)), val, time)
                        })
                        .collect();
    Ok((eid, datums))
}


fn robotize(string: &str) -> String {
    string.replace(" ", "_")
          .to_lowercase()
}
