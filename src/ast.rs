use data::Datum;

#[derive(Debug, Clone)]
pub enum Comparator {
    Contains,
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

impl Comparator {
    fn test_int(&self, left: u32, right: u32) -> bool {
        match self {
            &Comparator::Equal => left == right,
            &Comparator::Greater => left > right,
            &Comparator::GreaterOrEqual => left >= right,
            &Comparator::Less => left < right,
            &Comparator::LessOrEqual => left >= right,
            _ => false,
        }
    }

    fn test_str(&self, left: &str, right: &str) -> bool {
        match self {
            &Comparator::Contains => left.contains(right),
            &Comparator::Equal => left == right,
            &Comparator::Greater => left > right,
            &Comparator::GreaterOrEqual => left >= right,
            &Comparator::Less => left < right,
            &Comparator::LessOrEqual => left >= right,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionTest {
    e: Option<(u32, Comparator)>,
    a: Option<(String, Comparator)>,
    v: Option<(String, Comparator)>,
    t: Option<(u32, Comparator)>,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    True,
    Or(Box<ASTNode>, Box<ASTNode>),
    Expression(ExpressionTest),
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
            &ASTNode::Expression(ref exp) => {
                let e_pred = match &exp.e {
                    &Some((v, ref comp)) => comp.test_int(datum.e, v),
                    &None => true,
                };
                let a_pred = match &exp.a {
                    &Some((ref v, ref comp)) => comp.test_str(&datum.a, &v),
                    &None => true,
                };
                let v_pred = match &exp.v {
                    &Some((ref v, ref comp)) => comp.test_str(&datum.v, &v),
                    &None => true,
                };
                let t_pred = match &exp.t {
                    &Some((v, ref comp)) => comp.test_int(datum.t, v),
                    &None => true,
                };
                e_pred && a_pred && v_pred && t_pred
            }
            &ASTNode::Or(ref l, ref r) => l.eval(datum) || r.eval(datum),
        }
    }

    fn parse_expression(query: &str) -> Result<ASTNode, ParseError> {
        let exp_re = regex!(r"^[eavt][=:<>]=?\S+$");
        let mut exp = ExpressionTest {
            e: None,
            a: None,
            v: None,
            t: None,
        };

        for split in query.split(' ') {
            let predicate = split.trim();

            if exp_re.is_match(predicate) {
                let prefix = predicate.chars().nth(0).unwrap();
                let comparator_stop = if predicate.chars().nth(2).unwrap() == '=' { 3 } else { 2 };
                let val = &predicate[comparator_stop..];

                let comparator = match &predicate[1..comparator_stop] {
                    ":" => Comparator::Contains,
                    "=" => Comparator::Equal,
                    ">" => Comparator::Greater,
                    ">=" => Comparator::GreaterOrEqual,
                    "<" => Comparator::Less,
                    "<=" => Comparator::LessOrEqual,
                    _ => return Err(ParseError::InvalidPredicate(predicate)),
                };

                match prefix {
                    'e' => exp.e = Some((val.parse::<u32>().unwrap(), comparator)),
                    'a' => exp.a = Some((val.to_string(), comparator)),
                    'v' => exp.v = Some((val.to_string(), comparator)),
                    't' => exp.t = Some((val.parse::<u32>().unwrap(), comparator)),
                    _ => return Err(ParseError::InvalidState(query)),
                }
            } else {
                return Err(ParseError::InvalidPredicate(predicate));
            }
        }
        Ok(ASTNode::Expression(exp))
    }
}
