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
    pub fn test_int(&self, left: usize, right: usize) -> bool {
        match *self {
            Comparator::Equal => left == right,
            Comparator::Greater => left > right,
            Comparator::GreaterOrEqual => left >= right,
            Comparator::Less => left < right,
            Comparator::LessOrEqual => left >= right,
            _ => false,
        }
    }

    pub fn test_str(&self, left: &str, right: &str) -> bool {
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
    pub e: Option<(usize, Comparator)>,
    pub a: Option<(String, Comparator)>,
    pub v: Option<(String, Comparator)>,
    pub t: Option<(usize, Comparator)>,
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
}
