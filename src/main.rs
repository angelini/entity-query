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
mod filter;

use std::process;
use data::DB;
use ast::ASTNode;
use cli::CLICommand;
use filter::Filter;

fn main() {
    linenoise::history_set_max_len(1000);
    linenoise::history_load(".history");
    let mut db = DB::from_vec(vec![]);

    loop {
        match cli::read() {
            Ok(CLICommand::Query(query)) => {
                match ASTNode::parse(&query) {
                    Ok(ast) => {
                        let start = time::precise_time_s();
                        let res = Filter::new(&db, &ast, 12).execute();
                        println!("duration {}", time::precise_time_s() - start);
                        println!("len {}", res.datums.len());
                        println!("{}", res)
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(CLICommand::Load(filename)) => {
                let start = time::precise_time_s();
                match DB::from_file(&filename) {
                    Ok(d) => {
                        db.datums.extend(d.datums);
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
                        db.datums.extend(d.datums);
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
            Ok(CLICommand::Empty) => db = DB::from_vec(vec![]),
            Ok(CLICommand::None) => continue,
            Ok(CLICommand::Exit) => process::exit(0),
            Err(e) => println!("{:?}", e),
        }
    }
}
