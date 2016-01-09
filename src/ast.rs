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

#[derive(Debug, Clone, PartialEq)]
pub struct Predicates {
    pub e: Option<(usize, Comparator)>,
    pub a: Option<(String, Comparator)>,
    pub v: Option<(String, Comparator)>,
    pub t: Option<(usize, Comparator)>,
}

impl Predicates {
    pub fn new(e: Option<(usize, Comparator)>, a: Option<(String, Comparator)>,
               v: Option<(String, Comparator)>, t: Option<(usize, Comparator)>)
               -> Predicates {
        Predicates {
            e: e,
            a: a,
            v: v,
            t: t,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    True,
    Or(Box<AstNode>, Box<AstNode>),
    Expression(Predicates),
    Join(Predicates, Box<AstNode>),
    CachedJoin(Predicates, usize),
}

impl AstNode {
    pub fn parse(query: &str) -> Result<AstNode, grammar::ParseError> {
        grammar::ast(query)
    }

    pub fn from_parser(preds: Vec<(String, String, Option<AstNode>, Comparator)>) -> AstNode {
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
            AstNode::Join(Predicates {
                              e: e,
                              a: a,
                              v: v,
                              t: t,
                          },
                          child.unwrap())
        } else {
            AstNode::Expression(Predicates {
                e: e,
                a: a,
                v: v,
                t: t,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AstNode, Predicates, Comparator};

    #[test]
    fn parse_truthy() {
        let qs = [" ", "", "   "];
        let ast = AstNode::True;

        for q in &qs {
            assert_eq!(ast, AstNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_id_equality() {
        let qs = ["e=1", "   e=1 ", "e=1  "];
        let ast = AstNode::Expression(Predicates {
            e: Some((1, Comparator::Equal)),
            a: None,
            v: None,
            t: None,
        });

        for q in &qs {
            assert_eq!(ast, AstNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_multiple_predicates() {
        let qs = ["a=foo e=3 t=1  v=bar", " v=bar  t=1 ", "e=1 a=foo"];
        let asts = [AstNode::Expression(Predicates {
                        e: Some((3, Comparator::Equal)),
                        a: Some(("foo".to_owned(), Comparator::Equal)),
                        v: Some(("bar".to_owned(), Comparator::Equal)),
                        t: Some((1, Comparator::Equal)),
                    }),
                    AstNode::Expression(Predicates {
                        e: None,
                        a: None,
                        v: Some(("bar".to_owned(), Comparator::Equal)),
                        t: Some((1, Comparator::Equal)),
                    }),
                    AstNode::Expression(Predicates {
                        e: Some((1, Comparator::Equal)),
                        a: Some(("foo".to_owned(), Comparator::Equal)),
                        v: None,
                        t: None,
                    })];

        for (i, q) in qs.iter().enumerate() {
            assert_eq!(asts[i], AstNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_operators() {
        let qs = ["  a:foo e>=3 t<1 v<=bar ", "t>=1   e<1", "v:bar a>=foo "];
        let asts = [AstNode::Expression(Predicates {
                        e: Some((3, Comparator::GreaterOrEqual)),
                        a: Some(("foo".to_owned(), Comparator::Contains)),
                        v: Some(("bar".to_owned(), Comparator::LessOrEqual)),
                        t: Some((1, Comparator::Less)),
                    }),
                    AstNode::Expression(Predicates {
                        e: Some((1, Comparator::Less)),
                        a: None,
                        v: None,
                        t: Some((1, Comparator::GreaterOrEqual)),
                    }),
                    AstNode::Expression(Predicates {
                        e: None,
                        a: Some(("foo".to_owned(), Comparator::GreaterOrEqual)),
                        v: Some(("bar".to_owned(), Comparator::Contains)),
                        t: None,
                    })];

        for (i, q) in qs.iter().enumerate() {
            assert_eq!(asts[i], AstNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_or() {
        let qs = ["e=1 | e=2", "e=1 a:foo   |  e=2"];

        let e1 = Predicates::new(Some((1, Comparator::Equal)), None, None, None);
        let e2 = Predicates::new(Some((2, Comparator::Equal)), None, None, None);
        let e1afoo = Predicates::new(Some((1, Comparator::Equal)),
                                     Some(("foo".to_owned(), Comparator::Contains)),
                                     None,
                                     None);

        let asts = [AstNode::Or(Box::new(AstNode::Expression(e1)),
                                Box::new(AstNode::Expression(e2.clone()))),
                    AstNode::Or(Box::new(AstNode::Expression(e1afoo)),
                                Box::new(AstNode::Expression(e2)))];

        for (i, q) in qs.iter().enumerate() {
            assert_eq!(asts[i], AstNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_joins() {
        let qs = ["e:(a=foo/bar v=baz)", " e:(e=1)  a:other t=1 "];

        let e0 = Predicates::new(Some((0, Comparator::Contains)), None, None, None);
        let e1 = Predicates::new(Some((1, Comparator::Equal)), None, None, None);
        let e0aothert1 = Predicates::new(Some((0, Comparator::Contains)),
                                         Some(("other".to_owned(), Comparator::Contains)),
                                         None,
                                         Some((1, Comparator::Equal)));
        let afoovbaz = Predicates::new(None,
                                       Some(("foo/bar".to_owned(), Comparator::Equal)),
                                       Some(("baz".to_owned(), Comparator::Equal)),
                                       None);

        let asts = [AstNode::Join(e0, Box::new(AstNode::Expression(afoovbaz))),
                    AstNode::Join(e0aothert1, Box::new(AstNode::Expression(e1)))];

        for (i, q) in qs.iter().enumerate() {
            assert_eq!(asts[i], AstNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_nested_joins() {
        let q = "e:(e:(a=foo/bar v=baz) a:inside t>10) a:other";

        let e0aother = Predicates::new(Some((0, Comparator::Contains)),
                                       Some(("other".to_owned(), Comparator::Contains)),
                                       None,
                                       None);
        let e0ainsidet10 = Predicates::new(Some((0, Comparator::Contains)),
                                           Some(("inside".to_owned(), Comparator::Contains)),
                                           None,
                                           Some((10, Comparator::Greater)));
        let afoovbaz = Predicates::new(None,
                                       Some(("foo/bar".to_owned(), Comparator::Equal)),
                                       Some(("baz".to_owned(), Comparator::Equal)),
                                       None);


        let ast = AstNode::Join(e0aother,
                                Box::new(AstNode::Join(e0ainsidet10,
                                                       Box::new(AstNode::Expression(afoovbaz)))));

        assert_eq!(ast, AstNode::parse(q).unwrap());
    }
}
