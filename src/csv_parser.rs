use csv;
use data::{Datum, LoadError};

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
