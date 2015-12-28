extern crate regex;
extern crate rustc_serialize;
extern crate bincode;
extern crate csv;

mod data;
mod ast;
mod cli;

use data::DB;
use ast::ASTNode;
use cli::CLICommand;

fn main() {
    let mut db = DB::from_vec(vec![]);
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
                db = DB::from_file(&filename).unwrap();
                println!("{}", db)
            }
            Ok(CLICommand::LoadCSV(filename, entity, time)) => {
                db = DB::from_csv(&entity, &filename, &time).unwrap();
                println!("{}", db)
            }
            Ok(CLICommand::Write(filename)) => {
                let res = db.write(&filename);
                println!("res: {:?}", res)
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
