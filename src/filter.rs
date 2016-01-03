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
        let plan = Plan::new(&self.ast);
        let mut pool = Pool::new(self.threads as u32);
        let mut cache = Cache { executions: vec![] };

        for (i, step) in plan.steps.iter().enumerate() {
            if i < plan.steps.len() - 1 {
                let from_eids = Self::extract_eids(self.run_step(step, &cache, &mut pool));
                cache.executions.push(self.translate_eids(from_eids));
            }
        }

        DBView { datums: self.run_step(plan.steps.last().unwrap(), &cache, &mut pool) }
    }

    fn run_step(&self, ast: &ASTNode, cache: &Cache, pool: &mut Pool) -> Vec<&'a Datum> {
        let size = self.db.datums.len() / self.threads;
        let (tx, rx) = channel();

        pool.scoped(|scoped| {
            for i in 0..self.threads {
                let start = i * size;
                let stop = if i == (self.threads - 1) {
                    self.db.datums.len()
                } else {
                    i * size + size
                };

                let thread_tx = tx.clone();
                let thread_ast = ast.clone();
                let slice: &'a [Datum] = &self.db.datums[start..stop];

                scoped.execute(move || {
                    let results = slice.iter()
                                       .filter(|d| eval(&thread_ast, cache, d))
                                       .collect::<Vec<&Datum>>();
                    thread_tx.send(results).unwrap();
                })
            }
        });

        let mut results: Vec<&Datum> = vec![];
        for _ in 0..self.threads {
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
            let e_pred = match preds.e {
                Some((v, ref comp)) => comp.test_int(datum.e, v),
                None => true,
            };
            let a_pred = match preds.a {
                Some((ref v, ref comp)) => comp.test_str(&datum.a, &v),
                None => true,
            };
            let v_pred = match preds.v {
                Some((ref v, ref comp)) => comp.test_str(&datum.v, &v),
                None => true,
            };
            let t_pred = match preds.t {
                Some((v, ref comp)) => comp.test_int(datum.t, v),
                None => true,
            };
            e_pred && a_pred && v_pred && t_pred
        }
        ASTNode::CachedJoin(ref preds, cache_idx) => {
            let e_pred = match preds.e {
                Some((_, _)) => {
                    let eids = &cache.executions[cache_idx];
                    eids.contains(&datum.e)
                }
                None => true,
            };
            let a_pred = match preds.a {
                Some((ref v, ref comp)) => {
                    comp.test_str(&datum.a, &v)
                }
                None => true,
            };
            let v_pred = match preds.v {
                Some((ref v, ref comp)) => comp.test_str(&datum.v, &v),
                None => true,
            };
            let t_pred = match preds.t {
                Some((v, ref comp)) => comp.test_int(datum.t, v),
                None => true,
            };
            e_pred && a_pred && v_pred && t_pred
        }
        ASTNode::Or(ref l, ref r) => eval(l, cache, datum) || eval(r, cache, datum),
        ASTNode::Join(_, _) => unimplemented!(),
    }
}
