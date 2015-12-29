use data::Datum;

#[derive(Debug, Clone)]
pub enum ASTNode {
    True,
    Or(Box<ASTNode>, Box<ASTNode>),
    Expression {
        e: Option<u32>,
        a: Option<String>,
        v: Option<String>,
        t: Option<u32>,
    },
}

#[derive(Debug)]
pub enum ParseError<'a> {
    InvalidState(&'a str),
    InvalidPredicate(&'a str),
}

impl ASTNode {
    pub fn parse(query: &str) -> Result<ASTNode, ParseError> {
        let or_re = regex!(r"^(.*)\|(.*)$");
        let true_re = regex!(r"^\s*$");

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
            &ASTNode::Expression { e, ref a, ref v, t } => {
                let e_test = e.or(Some(datum.e)) == Some(datum.e);
                let t_test = t.or(Some(datum.t)) == Some(datum.t);
                let a_test = match a {
                    &Some(ref a) => datum.a.contains(a),
                    &None => true,
                };
                let v_test = match v {
                    &Some(ref v) => datum.v.contains(v),
                    &None => true,
                };

                e_test && a_test && v_test && t_test
            }
            &ASTNode::Or(ref l, ref r) => l.eval(datum) || r.eval(datum),
        }
    }

    fn parse_expression(query: &str) -> Result<ASTNode, ParseError> {
        let exp_re = regex!(r"^[eavt]:\S+$");
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
                            a: if prefix == "a:" { Some(val.to_string()) } else { a },
                            v: if prefix == "v:" { Some(val.to_string()) } else { v },
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
