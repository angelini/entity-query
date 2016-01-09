use std::collections::HashMap;
use ast::{AstNode, Predicates};

#[derive(Debug, PartialEq)]
pub enum IndexedNode {
    Base(Predicates),
    Or(usize, usize),
    Join(Predicates, usize),
}

#[derive(Debug)]
pub struct NewPlan {
    tasks: HashMap<usize, IndexedNode>,
    stages: Vec<Vec<usize>>,
}

impl NewPlan {
    pub fn new(ast: &AstNode) -> NewPlan {
        let mut stages: Vec<Vec<usize>> = vec![vec![]];
        let mut tasks = HashMap::new();
        let nodes = split_nodes(ast)
                        .into_iter()
                        .rev()
                        .enumerate()
                        .collect::<Vec<(usize, &AstNode)>>();

        for &(id, node) in &nodes {
            let indexed = index_node(node, &nodes);

            match indexed {
                IndexedNode::Base(_) => stages[0].push(id),
                IndexedNode::Join(_, c) => {
                    let child_idx = find_stage_idx(&stages, c);
                    let stage_idx = child_idx + 1;

                    if stages.len() < stage_idx - 1 {
                        stages.push(vec![]);
                    }
                    stages[stage_idx].push(id);
                }
                IndexedNode::Or(l, r) => {
                    let left_idx = find_stage_idx(&stages, l);
                    let right_idx = find_stage_idx(&stages, r);
                    let stage_idx = if left_idx > right_idx { left_idx + 1 } else { right_idx + 1 };

                    if stages.len() < stage_idx - 1 {
                        stages.push(vec![]);
                    }
                    stages[stage_idx].push(id);
                }
            }

            tasks.insert(id, indexed);
        }

        NewPlan {
            tasks: tasks,
            stages: stages,
        }
    }
}

fn find_stage_idx(stages: &[Vec<usize>], id: usize) -> usize {
    for (stage_idx, stage) in stages.iter().enumerate() {
        if stage.contains(&id) {
            return stage_idx;
        }
    }
    panic!("Can't find stage index")
}

fn index_node(ast: &AstNode, nodes: &[(usize, &AstNode)]) -> IndexedNode {
    match *ast {
        AstNode::True => IndexedNode::Base(Predicates::new(None, None, None, None)),
        AstNode::Expression(ref p) => IndexedNode::Base(p.clone()),
        AstNode::Join(ref p, ref c) => {
            let &(child_idx, _) = nodes.iter().find(|e| *e.1 == **c).unwrap();
            IndexedNode::Join(p.clone(), child_idx)
        }
        AstNode::Or(ref l, ref r) => {
            let &(left_idx, _) = nodes.iter().find(|e| *e.1 == **l).unwrap();
            let &(right_idx, _) = nodes.iter().find(|e| *e.1 == **r).unwrap();
            IndexedNode::Or(left_idx, right_idx)
        }
        _ => panic!(),
    }
}

fn split_nodes(ast: &AstNode) -> Vec<&AstNode> {
    vec![ast]
        .iter()
        .flat_map(|&ast| {
            match *ast {
                AstNode::True => vec![ast],
                AstNode::Expression(_) => vec![ast],
                AstNode::Or(ref l, ref r) => {
                    let mut ns = vec![ast];
                    ns.extend(split_nodes(l));
                    ns.extend(split_nodes(r));
                    ns
                }
                AstNode::Join(_, ref c) => {
                    let mut ns = vec![ast];
                    ns.extend(split_nodes(c));
                    ns
                }
                _ => panic!(),
            }
        })
        .collect()
}
