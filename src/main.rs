// http://carol-nichols.com/2015/12/09/rust-profiling-on-osx-cpu-time/
// #![feature(alloc_system)]
// extern crate alloc_system;

#![feature(plugin)]
#![plugin(regex_macros)]
#![plugin(peg_syntax_ext)]
#![plugin(clippy)]
#![allow(len_zero)] // for pegile macro

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
mod csv_parser;

use std::process;
use scoped_threadpool::Pool;

use ast::ASTNode;
use cli::CLICommand;
use csv_parser::CSVParser;
use data::DB;
use filter::Filter;

peg_file! grammar("grammar.rustpeg");

fn main() {
    linenoise::history_set_max_len(1000);
    linenoise::history_load(".history");

    let mut db = DB::new();
    let mut pool = Pool::new(12);

    loop {
        println!("size: {}", db.datums.len());
        match cli::read() {
            Ok(CLICommand::Query(query)) => {
                match ASTNode::parse(&query) {
                    Ok(ast) => {
                        println!("ast: {:?}", ast);
                        let start = time::precise_time_s();
                        let res = Filter::new(&db, &mut pool).execute(&ast);
                        println!("duration: {}", time::precise_time_s() - start);
                        println!("len: {}", res.datums.len());
                        println!("{}", res)
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(CLICommand::Load(filename)) => {
                let start = time::precise_time_s();
                db = DB::new(); // de-alloc the old DB
                match DB::from_file(&filename) {
                    Ok(d) => {
                        db = d;
                        println!("duration: {}", time::precise_time_s() - start);
                        println!("len: {}", db.datums.len());
                        println!("{}", db)
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(CLICommand::LoadCSV(filename, entity, time, joins)) => {
                let start = time::precise_time_s();
                let parser = CSVParser::new(&filename, &entity, &time, &joins);

                match parser.parse(&db, &mut pool) {
                    Ok((datums, refs, offset)) => {
                        println!("new: {}", datums.len());
                        db.datums.extend(datums);
                        db.refs.extend(refs);
                        db.offset += offset;
                        println!("duration: {}", time::precise_time_s() - start);
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
            Ok(CLICommand::Empty) => db = DB::new(),
            Ok(CLICommand::None) => continue,
            Ok(CLICommand::Exit) => process::exit(0),
            Err(e) => println!("{:?}", e),
        }
    }
}
