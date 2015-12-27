extern crate regex;
extern crate rustc_serialize;
extern crate bincode;
extern crate csv;

mod data;
mod ast;
mod cli;

use data::{DB, Datum};
use ast::ASTNode;
use cli::CLICommand;

fn robotize(string: &str) -> String {
    string.replace(" ", "_")
          .to_lowercase()
}

fn convert_csv(entity: &str, filename: &str, time: &str) -> DB {
    let mut rdr = csv::Reader::from_file(filename).unwrap();
    let headers = rdr.headers().expect("Headers required to convert CSV");

    let time_index = match headers.iter().enumerate().find(|&(_, h)| h == time) {
        Some((idx, _)) => idx,
        None => panic!("Time header not found {}", time),
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

    DB::new(datums)
}

fn main() {
    // let mut db = convert_csv("gdp", "data/gdp.csv", "Year");
    // let mut db = DB::from_vec(vec![]);
    let mut db = DB::from_file("data/gdp.db");
    // println!("db: {:?}", db.datums.iter().take(5).collect::<Vec<&Datum>>());
    println!("{}", db);

    loop {
        match cli::read() {
            Ok(CLICommand::Query(query)) => {
                match ASTNode::parse(&query) {
                    Ok(ast) => println!("{}", db.filter(|d| ast.eval(d))),
                    Err(e) => println!("err: {:?}", e),
                };
            }
            Ok(CLICommand::Load(filename)) => {
                db = DB::from_file(&filename);
                println!("{}", db)
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
