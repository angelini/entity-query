use std::sync::mpsc::channel;
use scoped_threadpool::Pool;
use ast::ASTNode;
use data::{Datum, DB, DBView};

pub struct Filter<'a> {
    threads: u32,
    db: &'a DB,
    ast: &'a ASTNode,
}

impl<'a> Filter<'a> {
    pub fn new(db: &'a DB, ast: &'a ASTNode, threads: u32) -> Filter<'a> {
        Filter {
            threads: threads,
            db: db,
            ast: ast,
        }
    }

    pub fn execute(self) -> DBView<'a> {
        let threads = 12;
        let size = self.db.datums.len() / self.threads as usize;
        let (tx, rx) = channel();
        let mut pool = Pool::new(threads as u32);

        pool.scoped(|scoped| {
            for i in 0..threads {
                let start = i * size;
                let stop = if i == (threads - 1) { self.db.datums.len() } else { i * size + size };

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
        for _ in 0..threads {
            results.extend(rx.recv().unwrap())
        }

        DBView { datums: results }
    }
}
