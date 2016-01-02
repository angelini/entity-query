use data::Datum;
use grammar;

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
pub struct ExpressionTest {
    e: Option<(u32, Comparator)>,
    a: Option<(String, Comparator)>,
    v: Option<(String, Comparator)>,
    t: Option<(u32, Comparator)>,
}

impl ExpressionTest {
    pub fn new(preds: Vec<(String, String, Comparator)>) -> ExpressionTest {
        let mut e = None;
        let mut a = None;
        let mut v = None;
        let mut t = None;

        for (name, val, comp) in preds {
            match name.as_ref() {
                "e" => e = Some((val.parse::<u32>().unwrap(), comp)),
                "a" => a = Some((val, comp)),
                "v" => v = Some((val, comp)),
                "t" => t = Some((val.parse::<u32>().unwrap(), comp)),
                _ => continue,
            }
        }

        ExpressionTest {
            e: e,
            a: a,
            v: v,
            t: t,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    True,
    Or(Box<ASTNode>, Box<ASTNode>),
    Expression(ExpressionTest),
}

impl ASTNode {
    pub fn parse(query: &str) -> Result<ASTNode, grammar::ParseError> {
        grammar::ast(query)
    }

    pub fn eval(&self, datum: &Datum) -> bool {
        match *self {
            ASTNode::True => true,
            ASTNode::Expression(ref exp) => {
                let e_pred = match exp.e {
                    Some((v, ref comp)) => comp.test_int(datum.e, v),
                    None => true,
                };
                let a_pred = match exp.a {
                    Some((ref v, ref comp)) => comp.test_str(&datum.a, &v),
                    None => true,
                };
                let v_pred = match exp.v {
                    Some((ref v, ref comp)) => comp.test_str(&datum.v, &v),
                    None => true,
                };
                let t_pred = match exp.t {
                    Some((v, ref comp)) => comp.test_int(datum.t, v),
                    None => true,
                };
                e_pred && a_pred && v_pred && t_pred
            }
            ASTNode::Or(ref l, ref r) => l.eval(datum) || r.eval(datum),
        }
    }
}
