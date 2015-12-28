extern crate regex;
extern crate rustc_serialize;
extern crate bincode;
extern crate csv;
extern crate linenoise;
extern crate flate2;
extern crate time;
extern crate scoped_threadpool;

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
                        let res = db.filter(&ast);
                        println!("duration {}", time::precise_time_s() - start);
                        println!("len {}", res.datums.len());
                        println!("{}", res)
                    }
                    Err(e) => println!("{:?}", e),
                };
            }
            Ok(CLICommand::Load(filename)) => {
                let start = time::precise_time_s();
                match DB::from_file(&filename) {
                    Ok(d) => {
                        db = d;
                        println!("duration {}", time::precise_time_s() - start);
                        println!("len {}", db.datums.len());
                        println!("{}", db)
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(CLICommand::LoadCSV(filename, entity, time)) => {
                let start = time::precise_time_s();
                match DB::from_csv(&entity, &filename, &time) {
                    Ok(d) => {
                        db = d;
                        println!("duration {}", time::precise_time_s() - start);
                        println!("len {}", db.datums.len());
                        println!("{}", db)
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(CLICommand::Write(filename)) => {
                match db.write(&filename) {
                    Ok(_) => println!("wrote: {}", filename),
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(CLICommand::Empty) => continue,
            Ok(CLICommand::Exit) => process::exit(0),
            Err(e) => println!("{:?}", e),
        }
    }
}
