extern crate regex;
extern crate rustc_serialize;
extern crate bincode;
extern crate csv;
extern crate linenoise;
extern crate flate2;
extern crate time;

mod data;
mod ast;
mod cli;

use std::process;
use data::DB;
use ast::ASTNode;
use cli::CLICommand;

fn main() {
    let mut db = DB::from_vec(vec![]);

    loop {
        match cli::read() {
            Ok(CLICommand::Query(query)) => {
                match ASTNode::parse(&query) {
                    Ok(ast) => {
                        let start = time::precise_time_s();
                        let res = db.filter(|d| ast.eval(d));
                        println!("duration {}", time::precise_time_s() - start);
                        println!("len {}", db.datums.len());
                        println!("{}", res)
                    }
                    Err(e) => println!("err: {:?}", e),
                };
            }
            Ok(CLICommand::Load(filename)) => {
                let start = time::precise_time_s();
                db = DB::from_file(&filename).unwrap();
                println!("duration {}", time::precise_time_s() - start);
                println!("len {}", db.datums.len());
                println!("{}", db)
            }
            Ok(CLICommand::LoadCSV(filename, entity, time)) => {
                let start = time::precise_time_s();
                db = DB::from_csv(&entity, &filename, &time).unwrap();
                println!("duration {}", time::precise_time_s() - start);
                println!("len {}", db.datums.len());
                println!("{}", db)
            }
            Ok(CLICommand::Write(filename)) => {
                let res = db.write(&filename);
                println!("res: {:?}", res)
            }
            Ok(CLICommand::Empty) => continue,
            Ok(CLICommand::Exit) => process::exit(0),
            Err(e) => println!("{:?}", e),
        }
    }
}
