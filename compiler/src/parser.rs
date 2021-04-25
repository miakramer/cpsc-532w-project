use nom::{IResult, branch::*, bytes::complete::tag, character::complete::*, combinator::*, error::{VerboseError, context}, multi::{many0, many1}, sequence::*};
use smol_str::SmolStr;

/* utilities */

// fn comment(i: &str) -> IResult<&str, (), VerboseError<&str>> {
//     value(
//         (),
//         pair(char(';'), is_not("\n\r"))
//     )(i)
// }

// macro_rules! space {
//     (w? $e:expr) => {
//         preceded(alt(whitespace0, pair(whitespace0, comment)), $e)
//     };
//     (w! $e:expr) => {
//         preceded(alt(whitespace1, pair(whitespace2, comment)), $e)
//     };
// }

/* c */

#[derive(Clone, Copy, Debug)]
pub enum C {
    Float(f64),
    Int(i128),
    Bool(bool),
}

pub fn decimal(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(many1(one_of("0123456789")))(input)
}

pub fn integer(input: &str) -> IResult<&str, i128, VerboseError<&str>> {
    // let (i, t) =
    map_res(
        recognize(pair(opt(tag("-")), many1(terminated(one_of("0123456789"), many0(char('_')))))),
        |out: &str| i128::from_str_radix(&str::replace(&out, "_", ""), 10),
    )(input)
}

pub fn float(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map_res(
        alt((
            // Case one: .42
            recognize(tuple((
                opt(tag("-")),
                char('.'),
                decimal,
                opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
            ))), // Case two: 42e42 and 42.42e42
            recognize(tuple((
                opt(tag("-")),
                decimal,
                opt(preceded(char('.'), decimal)),
                one_of("eE"),
                opt(one_of("+-")),
                decimal,
            ))), // Case three: 42. and 42.42
            recognize(tuple((opt(tag("-")), decimal, char('.'), opt(decimal)))),
        )),
        |res: &str| res.parse(),
    )(input)
}

pub fn bool(input: &str) -> IResult<&str, bool, VerboseError<&str>> {
    map_res(alt((tag("false"), tag("true"))), |res: &str| match res {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(()),
    })(input)
}

pub fn c(input: &str) -> IResult<&str, C, VerboseError<&str>> {
    let c_int = map(integer, |i| C::Int(i));
    let c_float = map(float, |f| C::Float(f));
    let c_bool = map(bool, |b| C::Bool(b));
    alt((c_float, c_int, c_bool))(input)
}

/* identifiers */

#[derive(Clone, Debug)]
pub enum Identifier {
    Newvar,
    Ident(SmolStr),
}

pub fn name(input: &str) -> IResult<&str, Identifier, VerboseError<&str>> {
    let (i, t) = recognize(pair(
        alpha1,
        many0(alt((alphanumeric1, tag("-")))),
    ))(input)?;

    Ok((i, Identifier::Ident(t.into())))
}

pub fn newvar(input: &str) -> IResult<&str, Identifier, VerboseError<&str>> {
    let (i, _t) = tag("_")(input)?;
    Ok((i, Identifier::Newvar))
}

pub fn identifier(input: &str) -> IResult<&str, Identifier, VerboseError<&str>> {
    alt((name, newvar))(input)
}

/* s expression */

macro_rules! s_expr {
    ($inner:expr) => {
        delimited(
            char('('),
            preceded(multispace0, $inner),
            context("closing paren", cut(preceded(multispace0, char(')')))),
        )
    };
}

macro_rules! b_expr {
    ($inner:expr) => {
        delimited(
            char('['),
            preceded(multispace0, $inner),
            context("closing paren", cut(preceded(multispace0, char(']')))),
        )
    };
}

/* relation */

#[derive(Clone, Copy, Debug)]
pub enum Relation {
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
}

impl Relation {
    pub fn pretty_print(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "≠",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Leq => "≤",
            Self::Geq => "≥"
        }
    }
}

pub fn relation(input: &str) -> IResult<&str, Relation, VerboseError<&str>> {
    map_res(
        alt((
            tag("="),
            tag("!="),
            tag("≠"),
            tag("<"),
            tag(">"),
            tag("<="),
            tag("≤"),
            tag(">="),
            tag("≥"),
        )),
        |res: &str| match res {
            "=" => Ok(Relation::Eq),
            "!=" | "≠" => Ok(Relation::Neq),
            "<" => Ok(Relation::Lt),
            ">" => Ok(Relation::Gt),
            "<=" | "≤" => Ok(Relation::Leq),
            ">=" | "≥" => Ok(Relation::Geq),
            _ => Err(()),
        },
    )(input)
}

/* domain-name */

#[derive(Clone, Copy, Debug)]
pub enum DomainName {
    IntRange,
    OneOf,
}

pub fn domain_name(input: &str) -> IResult<&str, DomainName, VerboseError<&str>> {
    map_res(
        alt((tag("int-range"), tag("one-of"))),
        |res: &str| match res {
            "one-of" => Ok(DomainName::OneOf),
            "int-range" => Ok(DomainName::IntRange),
            _ => Err(()),
        },
    )(input)
}

/* g */

#[derive(Clone, Copy, Debug)]
pub struct ProclaimThreshold(f64);

pub fn proclaim_threshold(input: &str) -> IResult<&str, ProclaimThreshold, VerboseError<&str>> {
    let inner = map(
        tuple((tag("proclaim-threshold"), multispace1, float)),
        |(_l, _w, r)| ProclaimThreshold(r),
    );
    // sexpr(inner)(input)
    context("", s_expr!(inner))(input)
}

/* e */

#[derive(Clone, Copy, Debug)]
pub enum Optimize {
    Maximize,
    Minimize,
}

#[derive(Clone, Debug)]
pub enum Expr {
    C(C),
    V(Identifier),
    Begin(Vec<Expr>),
    If {
        predicate: Box<Expr>,
        consequent: Box<Expr>,
        alternative: Box<Expr>,
    },
    Let {
        bindings: Vec<(Identifier, Expr)>,
        body: Box<Expr>,
    },
    Sample(Box<Expr>),
    Observe {
        observable: Box<Expr>,
        observed: Box<Expr>,
    },
    Decision(Box<Expr>),
    Constrain {
        relation: Relation,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // Optimize {
    //     which: Optimize,
    //     body: Box<Expr>,
    // },
    // Foreach {
    //     c: C,
    //     bindings: Vec<(Identifier, Expr)>,
    //     body: Box<Expr>,
    // },
    // Loop {
    //     c: C,
    //     base: Box<Expr>,
    //     f: Identifier,
    //     exprs: Vec<Expr>,
    // },
    Apply {
        head: Identifier,
        body: Vec<Expr>,
    },
}

pub fn parse_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    context(
        "expression",
        cut(alt((
            parse_expr_c,
            parse_expr_v,
            parse_begin,
            parse_if,
            parse_let,
            parse_sample,
            parse_observe,
            parse_decision,
            parse_constrain,
            parse_apply,
        ))),
    )(input)
}

pub fn parse_expr_c(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    context("c", map(c, |c| Expr::C(c)))(input)
}

pub fn parse_expr_v(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    context("variable", map(identifier, |i| Expr::V(i)))(input)
}

pub fn parse_begin(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let inner = map(
        tuple((
            tag("begin"),
            many1(preceded(multispace1, parse_expr)),
            multispace0,
        )),
        |(_head, body, _w)| Expr::Begin(body),
    );
    context("begin", s_expr!(inner))(input)
}

pub fn parse_if(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let inner = map(
        preceded(
            tag("if"),
            cut(tuple((
                context("(predicate)", preceded(multispace1, parse_expr)),
                context("(consequent)", preceded(multispace1, parse_expr)),
                context("(alternative)", preceded(multispace1, parse_expr)),
            ))),
        ),
        |(predicate, consequent, alternative)| Expr::If {
            predicate: Box::new(predicate),
            consequent: Box::new(consequent),
            alternative: Box::new(alternative),
        },
    );
    context("if", s_expr!(inner))(input)
}

macro_rules! varpair {
    () => {
        context(
            "binding pair",
            separated_pair(
                context("(name)", identifier),
                multispace1,
                context("(binding)", parse_expr),
            ),
        )
    };
}

pub fn parse_let(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let vars = b_expr!(tuple((
        varpair!(),
        many0(preceded(multispace1, varpair!()))
    )));
    let inner = map(
        preceded(
            tag("let"),
            cut(tuple((
                context("(bindings)", preceded(multispace1, vars)),
                context("(body)", preceded(multispace1, parse_expr)),
            ))),
        ),
        |(mut vars, expr)| {
            vars.1.insert(0, vars.0);
            Expr::Let {
                bindings: vars.1,
                body: Box::new(expr),
            }
        },
    );
    context("let", s_expr!(inner))(input)
}

pub fn parse_sample(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let inner = map(
        preceded(
            tag("sample"),
            cut(context("(sampled)", preceded(multispace1, parse_expr))),
        ),
        |expr| Expr::Sample(Box::new(expr)),
    );
    context("sample", s_expr!(inner))(input)
}

pub fn parse_observe(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let inner = map(
        preceded(
            tag("observe"),
            cut(tuple((
                context("(observable)", preceded(multispace1, parse_expr)),
                context("(observed)", preceded(multispace1, parse_expr)),
            ))),
        ),
        |(observable, observed)| Expr::Observe {
            observable: Box::new(observable),
            observed: Box::new(observed),
        },
    );
    context("observe", s_expr!(inner))(input)
}

pub fn parse_decision(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let inner = map(
        preceded(tag("decision"), cut(preceded(multispace1, parse_expr))),
        |expr| Expr::Decision(Box::new(expr)),
    );
    context("decision", s_expr!(inner))(input)
}

pub fn parse_constrain(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let inner = map(
        preceded(
            tag("constrain"),
            cut(tuple((
                preceded(multispace1, relation),
                preceded(multispace1, parse_expr),
                preceded(multispace1, parse_expr),
            ))),
        ),
        |(relation, left, right)| Expr::Constrain {
            relation,
            left: Box::new(left),
            right: Box::new(right),
        },
    );
    context("constrain", s_expr!(inner))(input)
}

pub fn function_identifier(input: &str) -> IResult<&str, Identifier, VerboseError<&str>> {
    map(pair(
        many1(one_of("?!+-*/=≠<>≤≥-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")), 
        many0(one_of("?!+-*/=≠<>≤≥-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"))),
        |(l, r)| {
            let mut string = String::with_capacity(l.len() + r.len());
            string.extend(l.into_iter());
            string.extend(r.into_iter());
            Identifier::Ident(string.into())
        })(input)
}

pub fn parse_apply(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let inner = cut(map(
        tuple((function_identifier, many0(preceded(multispace1, parse_expr)), multispace0)),
        |(head, body, _w)| Expr::Apply { head, body },
    ));
    context("apply", s_expr!(inner))(input)
}

/* q */

#[derive(Clone, Debug)]
pub struct Defn {
    pub name: Identifier,
    pub args: Vec<Identifier>,
    pub body: Expr,
}

pub fn parse_defn(input: &str) -> IResult<&str, Defn, VerboseError<&str>> {
    let inner = map(
        preceded(
            tag("defn"),
            cut(tuple((
                context("(name)", preceded(multispace1, identifier)),
                context(
                    "(args)",
                    preceded(
                        multispace1,
                        b_expr!(many0(preceded(multispace0, identifier))),
                    ),
                ),
                context("(body)", preceded(multispace1, parse_expr)),
            ))),
        ),
        |(name, args, body)| Defn { name, args, body },
    );
    context("defn", s_expr!(inner))(input)
}

/* prog */

#[derive(Clone, Debug)]
pub struct Program {
    pub proclaim: ProclaimThreshold,
    pub defns: Vec<Defn>,
    pub body: Expr,
}

pub fn parse_program(input: &str) -> IResult<&str, Program, VerboseError<&str>> {
    cut(context(
        "(top-level)",
        map(
            tuple((
                context("(proclaim)", preceded(multispace0, proclaim_threshold)),
                context("(defns)", many0(preceded(multispace0, parse_defn))),
                context("(main)", preceded(multispace0, parse_expr)),
            )),
            |(proclaim, defns, body)| Program {
                proclaim,
                defns,
                body,
            },
        )),
    )(input)
}
