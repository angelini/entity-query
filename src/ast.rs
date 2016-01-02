use data::Datum;
use grammar;

#[derive(Debug, Clone, PartialEq)]
pub enum Comparator {
    Contains,
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

impl Comparator {
    fn test_int(&self, left: usize, right: usize) -> bool {
        match *self {
            Comparator::Equal => left == right,
            Comparator::Greater => left > right,
            Comparator::GreaterOrEqual => left >= right,
            Comparator::Less => left < right,
            Comparator::LessOrEqual => left >= right,
            _ => false,
        }
    }

    fn test_str(&self, left: &str, right: &str) -> bool {
        match *self {
            Comparator::Contains => left.contains(right),
            Comparator::Equal => left == right,
            Comparator::Greater => left > right,
            Comparator::GreaterOrEqual => left >= right,
            Comparator::Less => left < right,
            Comparator::LessOrEqual => left >= right,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Predicates {
    e: Option<(usize, Comparator)>,
    a: Option<(String, Comparator)>,
    v: Option<(String, Comparator)>,
    t: Option<(usize, Comparator)>,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    True,
    Or(Box<ASTNode>, Box<ASTNode>),
    Expression(Predicates),
    Join(Predicates, Box<ASTNode>),
    CachedJoin(Predicates, usize),
}

impl ASTNode {
    pub fn parse(query: &str) -> Result<ASTNode, grammar::ParseError> {
        grammar::ast(query)
    }

    pub fn from_parser(preds: Vec<(String, String, Option<ASTNode>, Comparator)>) -> ASTNode {
        let mut e = None;
        let mut a = None;
        let mut v = None;
        let mut t = None;
        let mut child = None;

        for (name, val, ast, comp) in preds {
            match name.as_ref() {
                "e" => {
                    if comp == Comparator::Contains {
                        e = Some((0, comp));
                        child = Some(Box::new(ast.unwrap()));
                    } else {
                        e = Some((val.parse::<usize>().unwrap(), comp))
                    }
                }
                "a" => a = Some((val, comp)),
                "v" => v = Some((val, comp)),
                "t" => t = Some((val.parse::<usize>().unwrap(), comp)),
                _ => continue,
            }
        }

        if child.is_some() {
            ASTNode::Join(Predicates {
                              e: e,
                              a: a,
                              v: v,
                              t: t,
                          },
                          child.unwrap())
        } else {
            ASTNode::Expression(Predicates {
                e: e,
                a: a,
                v: v,
                t: t,
            })
        }
    }


    pub fn eval(&self, datum: &Datum) -> bool {
        match *self {
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
            ASTNode::Join(ref preds, ref child) => {
                unimplemented!()
            }
            ASTNode::CachedJoin(ref preds, cache_idx) => {
                unimplemented!()
            }
            ASTNode::Or(ref l, ref r) => l.eval(datum) || r.eval(datum),
        }
    }
}
