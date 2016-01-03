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

#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::{ASTNode, Predicates, Comparator};

    #[test]
    fn parse_truthy() {
        let qs = [" ", "", "   "];
        let ast = ASTNode::True;

        for q in &qs {
            assert_eq!(ast, ASTNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_id_equality() {
        let qs = ["e=1", "   e=1 ", "e=1  "];
        let ast = ASTNode::Expression(Predicates {
            e: Some((1, Comparator::Equal)),
            a: None,
            v: None,
            t: None,
        });

        for q in &qs {
            assert_eq!(ast, ASTNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_multiple_predicates() {
        let qs = ["a=foo e=3 t=1  v=bar", " v=bar  t=1 ", "e=1 a=foo"];
        let asts = [ASTNode::Expression(Predicates {
                        e: Some((3, Comparator::Equal)),
                        a: Some(("foo".to_owned(), Comparator::Equal)),
                        v: Some(("bar".to_owned(), Comparator::Equal)),
                        t: Some((1, Comparator::Equal)),
                    }),
                    ASTNode::Expression(Predicates {
                        e: None,
                        a: None,
                        v: Some(("bar".to_owned(), Comparator::Equal)),
                        t: Some((1, Comparator::Equal)),
                    }),
                    ASTNode::Expression(Predicates {
                        e: Some((1, Comparator::Equal)),
                        a: Some(("foo".to_owned(), Comparator::Equal)),
                        v: None,
                        t: None,
                    })];

        for (i, q) in qs.iter().enumerate() {
            assert_eq!(asts[i], ASTNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_operators() {
        let qs = ["  a:foo e>=3 t<1 v<=bar ", "t>=1   e<1", "v:bar a>=foo "];
        let asts = [ASTNode::Expression(Predicates {
                        e: Some((3, Comparator::GreaterOrEqual)),
                        a: Some(("foo".to_owned(), Comparator::Contains)),
                        v: Some(("bar".to_owned(), Comparator::LessOrEqual)),
                        t: Some((1, Comparator::Less)),
                    }),
                    ASTNode::Expression(Predicates {
                        e: Some((1, Comparator::Less)),
                        a: None,
                        v: None,
                        t: Some((1, Comparator::GreaterOrEqual)),
                    }),
                    ASTNode::Expression(Predicates {
                        e: None,
                        a: Some(("foo".to_owned(), Comparator::GreaterOrEqual)),
                        v: Some(("bar".to_owned(), Comparator::Contains)),
                        t: None,
                    })];

        for (i, q) in qs.iter().enumerate() {
            assert_eq!(asts[i], ASTNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_joins() {
        let qs = ["e:(a=foo/bar v=baz)", " e:(e=1)  a:other t=1 "];
        let asts = [ASTNode::Join(Predicates {
                                      e: Some((0, Comparator::Contains)),
                                      a: None,
                                      v: None,
                                      t: None,
                                  },
                                  Box::new(ASTNode::Expression(Predicates {
                                      e: None,
                                      a: Some(("foo/bar".to_owned(), Comparator::Equal)),
                                      v: Some(("baz".to_owned(), Comparator::Equal)),
                                      t: None,
                                  }))),
                    ASTNode::Join(Predicates {
                                      e: Some((0, Comparator::Contains)),
                                      a: Some(("other".to_owned(), Comparator::Contains)),
                                      v: None,
                                      t: Some((1, Comparator::Equal)),
                                  },
                                  Box::new(ASTNode::Expression(Predicates {
                                      e: Some((1, Comparator::Equal)),
                                      a: None,
                                      v: None,
                                      t: None,
                                  })))];

        for (i, q) in qs.iter().enumerate() {
            assert_eq!(asts[i], ASTNode::parse(q).unwrap());
        }
    }

    #[test]
    fn parse_nested_joins() {
        let q = "e:(e:(a=foo/bar v=baz) a:inside t>10) a:other";
        let ast =
            ASTNode::Join(Predicates {
                              e: Some((0, Comparator::Contains)),
                              a: Some(("other".to_owned(), Comparator::Contains)),
                              v: None,
                              t: None,
                          },
                          Box::new(ASTNode::Join(Predicates {
                                                     e: Some((0, Comparator::Contains)),
                                                     a: Some(("inside".to_owned(),
                                                              Comparator::Contains)),
                                                     v: None,
                                                     t: Some((10, Comparator::Greater)),
                                                 },
                                                 Box::new(ASTNode::Expression(Predicates {
                                                     e: None,
                                                     a: Some(("foo/bar".to_owned(),
                                                              Comparator::Equal)),
                                                     v: Some(("baz".to_owned(), Comparator::Equal)),
                                                     t: None,
                                                 })))));

        assert_eq!(ast, ASTNode::parse(q).unwrap());
    }
}
