#[derive(Debug)]
pub struct Datum<'l> {
    pub e: u32,
    pub a: &'l str,
    pub v: &'l str,
    pub t: u32,
}

impl<'l> Datum<'l> {
    pub fn new(e: u32, a: &'l str, v: &'l str, t: u32) -> Datum<'l> {
        Datum {
            e: e,
            a: a,
            v: v,
            t: t,
        }
    }
}

#[derive(Debug)]
pub struct DB<'a> {
    datums: Vec<Datum<'a>>,
}

impl<'a> DB<'a> {
    pub fn from_vec(datums: Vec<(u32, &'a str, &'a str, u32)>) -> DB<'a> {
        DB { datums: datums.iter().map(|t| Datum::new(t.0, t.1, t.2, t.3)).collect::<Vec<Datum>>() }
    }

    pub fn filter<F>(self, predicate: F) -> DB<'a>
        where F: Fn(&Datum) -> bool
    {
        DB { datums: self.datums.into_iter().filter(|d| predicate(d)).collect::<Vec<Datum>>() }
    }
}
