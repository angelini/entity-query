mod data;
mod ast;
mod cli;

extern crate regex;

use data::DB;
use ast::ASTNode;

fn main() {
    let query = "a:fo";
    let db = DB::from_vec(vec![(1, "entity/foo", "foo-val-1", 1),
                               (1, "entity/bar", "bar-val-1", 2),
                               (2, "entity/foo", "foo-val-2", 3),
                               (2, "entity/bar", "bar-val-2", 4),
                               (3, "entity/foo", "foo-val-3", 5),
                               (3, "entity/bar", "bar-val-3", 6)]);
    println!("db {:?}", db);

    let ast = match ASTNode::parse(query) {
        Ok(ast) => ast,
        Err(e) => panic!("query: {:?}\nerr: {:?}", query, e),
    };

    let result = db.filter(|d| ast.eval(&d));
    println!("res {:?}", result);

    loop {
        let command = cli::read();
        println!("command {:?}", command)
    }
}
