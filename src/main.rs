extern crate regex;
extern crate rustc_serialize;
extern crate bincode;

mod data;
mod ast;
mod cli;

use data::DB;
use ast::ASTNode;
use cli::CLICommand;

fn main() {
    let mut db = DB::from_vec(vec![]);
    println!("db {:?}", db);

    loop {
        match cli::read() {
            Ok(CLICommand::Query(query)) => {
                match ASTNode::parse(&query) {
                    Ok(ast) => println!("{:?}", db.filter(|d| ast.eval(d))),
                    Err(e) => println!("err: {:?}", e),
                };
            }
            Ok(CLICommand::Load(filename)) => {
                db = DB::from_file(&filename);
                println!("db {:?}", db)
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
