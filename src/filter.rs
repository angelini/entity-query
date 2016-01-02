use std::sync::mpsc::channel;
use scoped_threadpool::Pool;
use ast::ASTNode;
use data::{Datum, DB, DBView};

pub struct Filter<'a> {
    threads: usize,
    db: &'a DB,
    ast: &'a ASTNode,
}

impl<'a> Filter<'a> {
    pub fn new(db: &'a DB, ast: &'a ASTNode, threads: usize) -> Filter<'a> {
        Filter {
            threads: threads,
            db: db,
            ast: ast,
        }
    }

    pub fn execute(self) -> DBView<'a> {
        let size = self.db.datums.len() / self.threads;
        let (tx, rx) = channel();
        let mut pool = Pool::new(self.threads as u32);

        println!("plan: {:?}", Plan::new(&self.ast).steps);

        pool.scoped(|scoped| {
            for i in 0..self.threads {
                let start = i * size;
                let stop = if i == (self.threads - 1) { self.db.datums.len() } else { i * size + size };

                let thread_tx = tx.clone();
                let thread_ast = self.ast.clone();
                let slice: &'a [Datum] = &self.db.datums[start..stop];

                scoped.execute(move || {
                    let results = slice.iter()
                                       .filter(|d| thread_ast.eval(d))
                                       .collect::<Vec<&Datum>>();
                    thread_tx.send(results).unwrap();
                })
            }
        });

        let mut results: Vec<&Datum> = vec![];
        for _ in 0..self.threads {
            results.extend(rx.recv().unwrap())
        }

        DBView { datums: results }
    }
}

struct Plan {
    steps: Vec<ASTNode>,
}

impl Plan {
    fn new(ast: &ASTNode) -> Plan {
        Plan { steps: Self::expand(ast) }
    }

    fn expand(ast: &ASTNode) -> Vec<ASTNode> {
        match *ast {
            ASTNode::Join(ref p, ref c) => {
                let mut expanded = Plan::expand(c);
                let len = expanded.len();
                expanded.push(ASTNode::CachedJoin(p.clone(), len - 1));
                expanded
            }
            ASTNode::Or(ref l, ref r) => {
                let mut expanded = Plan::expand(l);
                expanded.append(&mut Plan::expand(r));
                expanded
            }
            _ => vec![ast.clone()]
        }
    }
}
