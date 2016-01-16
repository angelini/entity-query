use rayon;
use std::collections::{HashSet, HashMap};
use std::thread;
use std::sync::{Arc, mpsc};

use ast::{AstNode, Comparator};
use data::{Datum, Db, DbView};
use plan::{Plan, Task, Action, IndexedNode};

type Tags = HashSet<usize>;

type Cache = HashMap<usize, Vec<usize>>;

pub fn exec<'a>(ast: &'a AstNode, datums: &'a [&'a Datum]) -> Vec<&'a Datum> {
    let plan = Plan::new(ast);

    let mut cache = Cache::new();
    let mut tagged_datums = datums.into_iter()
        .map(|d| (*d, HashSet::new()))
        .collect::<Vec<(&'a Datum, Tags)>>();

    for stage in &plan.stages {
        let current_tasks = stage.iter()
                                 .map(|id| (*id, plan.tasks.get(id).unwrap().clone()))
                                 .collect::<Vec<(usize, Task)>>();

        exec_stage(&mut tagged_datums, &mut cache, &current_tasks);
    }

    tagged_datums.into_iter().map(|(d, _)| d).collect::<Vec<&'a Datum>>()
}

fn exec_stage(datums: &mut [(&Datum, Tags)], cache: &mut Cache, tasks: &[(usize, Task)]) {
    let mid = datums.len() / 2;

    if datums.len() > 1 {
        let (lo, hi) = datums.split_at_mut(mid);
        rayon::join(move || exec_stage(lo, &mut cache, tasks),
                    move || exec_stage(hi, &mut cache, tasks));
    } else if datums.len() == 1 {
        let row = &mut datums[0];
        exec_tasks(row.0, &mut row.1, &mut cache, tasks);
    }
}

fn exec_tasks(datum: &Datum, tags: &mut Tags, cache: &mut Cache, tasks: &[(usize, Task)]) {
    for &(id, ref task) in tasks {
        match *task {
            Task(_, Action::Collect) => unimplemented!(),
            Task(ref node, Action::Tag) => {
                if test_node(&node, datum, &*tags, &*cache) {
                    tags.insert(id);
                }
            }
        }
    }
}

fn test_node(node: &IndexedNode, datum: &Datum, tags: &Tags, cache: &Cache) -> bool {
    match *node {
        IndexedNode::Base(ref preds) => {
            test_predicate(&preds.e, datum.e) && test_predicate_with_contains(&preds.a, &datum.a) &&
            test_predicate_with_contains(&preds.v, &datum.v) &&
            test_predicate(&preds.t, datum.t)
        },
        IndexedNode::Or(ref l, ref r) => {
            tags.contains(l) && tags.contains(r)
        },
        IndexedNode::Join(ref preds, ref c) => {
            test_join_predicate(&preds.e, cache.get(c).unwrap(), datum.e) &&
            test_predicate_with_contains(&preds.a, &datum.a) &&
            test_predicate_with_contains(&preds.v, &datum.v) &&
            test_predicate(&preds.t, datum.t)
        }
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
