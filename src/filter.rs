use std::sync::mpsc::channel;
use scoped_threadpool::Pool;

use ast::{ASTNode, Comparator};
use data::{Datum, DB, DBView};

pub struct Filter<'a> {
    db: &'a DB,
    pool: &'a mut Pool,
}

impl<'a> Filter<'a> {
    pub fn new(db: &'a DB, pool: &'a mut Pool) -> Filter<'a> {
        Filter {
            db: db,
            pool: pool,
        }
    }

    pub fn execute(self, ast: &ASTNode) -> DBView<'a> {
        let plan = Plan::new(ast);
        let mut cache = Cache { executions: vec![] };

        for (i, step) in plan.steps.iter().enumerate() {
            if i < plan.steps.len() - 1 {
                let from_datums = Self::run_step(self.db, self.pool, &cache, step);
                let from_eids = Self::extract_eids(from_datums);
                cache.executions.push(self.translate_eids(from_eids));
            }
        }

        DBView { datums: Self::run_step(self.db, self.pool, &cache, plan.steps.last().unwrap()) }
    }

    fn translate_eids(&self, eids: Vec<usize>) -> Vec<usize> {
        self.db
            .refs
            .iter()
            .map(|db_ref| {
                for eid in &eids {
                    if *eid == db_ref.e {
                        return Some(db_ref.v);
                    } else if *eid == db_ref.v {
                        return Some(db_ref.e);
                    }
                }
                None
            })
            .filter(|db_ref| db_ref.is_some())
            .map(|db_ref| db_ref.unwrap())
            .collect()
    }

    fn run_step(db: &'a DB, pool: &mut Pool, cache: &Cache, ast: &ASTNode) -> Vec<&'a Datum> {
        let db_size = db.datums.len();
        let threads = pool.thread_count() as usize;
        let size = db.datums.len() / threads;
        let (tx, rx) = channel();

        pool.scoped(|scoped| {
            for i in 0..threads {
                let start = i * size;
                let stop = if i == (threads - 1) { db_size } else { i * size + size };

                let thread_tx = tx.clone();
                let thread_ast = ast.clone();
                let slice: &'a [Datum] = &db.datums[start..stop];

                scoped.execute(move || {
                    let results = slice.iter()
                                       .filter(|d| eval(&thread_ast, cache, d))
                                       .collect::<Vec<&Datum>>();
                    thread_tx.send(results).unwrap();
                })
            }
        });

        let mut results: Vec<&Datum> = vec![];
        for _ in 0..threads {
            results.extend(rx.recv().unwrap())
        }
        results
    }

    fn extract_eids(datums: Vec<&Datum>) -> Vec<usize> {
        let mut eids: Vec<usize> = datums.into_iter().map(|d| d.e).collect::<Vec<usize>>();
        eids.sort();
        eids.dedup();
        eids
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
            _ => vec![ast.clone()],
        }
    }
}

struct Cache {
    executions: Vec<Vec<usize>>,
}

fn eval(ast: &ASTNode, cache: &Cache, datum: &Datum) -> bool {
    match *ast {
        ASTNode::True => true,
        ASTNode::Expression(ref preds) => {
            test_predicate(&preds.e, datum.e) && test_predicate_with_contains(&preds.a, &datum.a) &&
            test_predicate_with_contains(&preds.v, &datum.v) &&
            test_predicate(&preds.t, datum.t)
        }

        ASTNode::CachedJoin(ref preds, cache_idx) => {
            test_join_predicate(&preds.e, &cache.executions[cache_idx], datum.e) &&
            test_predicate_with_contains(&preds.a, &datum.a) &&
            test_predicate_with_contains(&preds.v, &datum.v) &&
            test_predicate(&preds.t, datum.t)
        }
        ASTNode::Or(ref l, ref r) => eval(l, cache, datum) || eval(r, cache, datum),
        ASTNode::Join(_, _) => unimplemented!(),
    }
}

fn test_predicate(pred: &Option<(usize, Comparator)>, datum_val: usize) -> bool {
    match *pred {
        Some((v, ref comp)) => comp.test_int(datum_val, v),
        None => true,
    }
}

fn test_predicate_with_contains(pred: &Option<(String, Comparator)>, datum_val: &str) -> bool {
    match *pred {
        Some((ref v, ref comp)) => comp.test_str(datum_val, &v),
        None => true,
    }
}

fn test_join_predicate(pred: &Option<(usize, Comparator)>, eids: &[usize], datum_val: usize) -> bool {
    match *pred {
        Some(_) => eids.contains(&datum_val),
        None => true,
    }
}
