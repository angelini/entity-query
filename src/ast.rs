use regex::Regex;
use data::Datum;

#[derive(Debug)]
pub enum ASTNode<'a> {
    True,
    Or(Box<ASTNode<'a>>, Box<ASTNode<'a>>),
    Expression {
        e: Option<u32>,
        a: Option<&'a str>,
        v: Option<&'a str>,
        t: Option<u32>,
    },
}

#[derive(Debug)]
pub enum ParseError<'a> {
    InvalidState(&'a str),
    InvalidPredicate(&'a str),
}

impl<'a> ASTNode<'a> {
    pub fn parse(query: &str) -> Result<ASTNode, ParseError> {
        let or_re = Regex::new(r"^(.*)\|(.*)$").unwrap();
        let true_re = Regex::new(r"^\s*$").unwrap();

        if let Some(caps) = or_re.captures(query) {
            let left = Self::parse(caps.at(1).unwrap());
            let right = Self::parse(caps.at(2).unwrap());

            match (left, right) {
                (Ok(l), Ok(r)) => Ok(ASTNode::Or(Box::new(l), Box::new(r))),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),

            }
        } else if true_re.is_match(query) {
            Ok(ASTNode::True)
        } else {
            Self::parse_expression(query)
        }
    }

    pub fn eval(&self, datum: &Datum) -> bool {
        match self {
            &ASTNode::True => true,
            &ASTNode::Expression { e, a, v, t } => {
                let e_test = e.or(Some(datum.e)) == Some(datum.e);
                let a_test = datum.a.contains(a.or(Some(datum.a)).unwrap());
                let v_test = datum.v.contains(v.or(Some(datum.v)).unwrap());
                let t_test = t.or(Some(datum.t)) == Some(datum.t);
                e_test && a_test && v_test && t_test
            }
            &ASTNode::Or(ref l, ref r) => l.eval(datum) || r.eval(datum),
        }
    }

    fn parse_expression(query: &str) -> Result<ASTNode, ParseError> {
        let exp_re = Regex::new(r"^[eavt]:\S+$").unwrap();
        let mut exp = ASTNode::Expression {
            e: None,
            a: None,
            v: None,
            t: None,
        };

        for split in query.split(' ') {
            let predicate = split.trim();

            if exp_re.is_match(predicate) {
                let prefix = &predicate[..2];
                let val = &predicate[2..];

                exp = match exp {
                    ASTNode::Expression { e, a, v, t } => {
                        ASTNode::Expression {
                            e: if prefix == "e:" { Some(val.parse::<u32>().unwrap()) } else { e },
                            a: if prefix == "a:" { Some(val) } else { a },
                            v: if prefix == "v:" { Some(val) } else { v },
                            t: if prefix == "t:" { Some(val.parse::<u32>().unwrap()) } else { t },
                        }
                    }
                    _ => return Err(ParseError::InvalidState(query)),
                }
            } else {
                return Err(ParseError::InvalidPredicate(predicate));
            }
        }
        Ok(exp)
    }
}
