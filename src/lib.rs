#![feature(field_init_shorthand)]
#![feature(conservative_impl_trait)]
#![feature(pattern)]

#[macro_use]
extern crate peresil;

use std::collections::BTreeSet;

// define what you want to parse; likely a string
// create an error type
// definte type aliases
type Point<'s> = peresil::StringPoint<'s>;
type Master<'s> = peresil::ParseMaster<Point<'s>, Error>;
type Progress<'s, T> = peresil::Progress<Point<'s>, T, Error>;

// define an error type - emphasis on errors. Need to implement Recoverable (more to discuss.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Error {
    Literal(&'static str),
    IdentNotFound,
}

impl peresil::Recoverable for Error {
    fn recoverable(&self) -> bool { true }
}

// Construct a point, initialize  the master. This is what stores errors
// todo: rename?

pub fn parse_rust_file(file: &str) {
    let mut pt = Point::new(file);
    let mut pm = Master::new();

    loop {
        let next_pt;

        let top_level = top_level(&mut pm, pt);
        let top_level = pm.finish(top_level);

        match top_level.status {
            peresil::Status::Success(s) => {
                println!("Ok {:#?}", s);
                next_pt = top_level.point;
            },
            peresil::Status::Failure(e) => {
                println!("Err @ {}: {:?}", top_level.point.offset, e.into_iter().collect::<BTreeSet<_>>());
                println!(">>{}<<", &file[top_level.point.offset..]);
                break;
            },
        }

        if next_pt.offset <= pt.offset {
            let end = std::cmp::min(pt.offset + 10, file.len());
            panic!("Could not make progress: {}...", &file[pt.offset..end]);
        }
        pt = next_pt;

        if pt.s.is_empty() { break }
    }

    // TODO: add `expect` to progress?
}

// TODO: enum variants track whole extent, enum delegates

type Extent = (usize, usize);

#[derive(Debug)]
enum TopLevel {
    Comment(Extent),
    Function(Function),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    Attribute(Extent),
    ExternCrate(Crate),
    Use(Use),
    TypeAlias(TypeAlias),
    Whitespace(Extent),
}

#[derive(Debug)]
struct Use {
    extent: Extent,
    name: Extent,
}

#[derive(Debug)]
struct Function {
    extent: Extent,
    header: FunctionHeader,
    body: Block,
}

#[derive(Debug)]
struct FunctionHeader {
    extent: Extent,
    visibility: Option<Extent>,
    name: Extent,
    generics: Vec<Generic>,
    arguments: Vec<Argument>,
    return_type: Option<Type>,
    wheres: Vec<Where>,
}

//#[derive(Debug)]
type Generic = Extent;

type Type = Extent;

fn ex(start: Point, end: Point) -> Extent {
    let ex = (start.offset, end.offset);
    assert!(ex.1 > ex.0, "{} does not come before {}", ex.1, ex.0);
    ex
}

#[derive(Debug)]
struct Enum {
    extent: Extent,
    name: Extent,
    variants: Vec<EnumVariant>,
}

#[derive(Debug)]
struct EnumVariant {
    extent: Extent,
    name: Extent,
    body: Vec<EnumVariantBody>,
}

type EnumVariantBody = Extent;

#[derive(Debug)]
enum Argument {
    SelfArgument,
    Named { name: Extent, typ: Type }
}

#[derive(Debug)]
struct Where {
    name: Type,
    bounds: Extent,
}

#[derive(Debug)]
struct Block {
    extent: Extent,
    statements: Vec<Statement>,
    expression: Option<Expression>,
}

#[derive(Debug)]
enum Statement {
    Explicit(Expression),
    Implicit(Expression),
}

impl Statement {
    #[allow(dead_code)]
    fn explicit(self) -> Option<Expression> {
        match self {
            Statement::Explicit(e) => Some(e),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn implicit(self) -> Option<Expression> {
        match self {
            Statement::Implicit(e) => Some(e),
            _ => None,
        }
    }

    fn is_implicit(&self) -> bool {
        match *self {
            Statement::Implicit(..) => true,
            _ => false
        }
    }
}

#[derive(Debug)]
struct Expression {
    extent: Extent,
    kind: ExpressionKind,
}

#[derive(Debug)]
enum ExpressionKind {
    MacroCall(MacroCall),
    Let(Let),
    Assign(Assign),
    Tuple(Tuple),
    FieldAccess(FieldAccess),
    Value(Value),
    Block(Box<Block>),
    FunctionCall(FunctionCall),
    MethodCall(MethodCall),
    Loop(Loop),
    Binary(Binary),
    If(If),
    Match(Match),
    True,
}

#[derive(Debug)]
struct MacroCall {
    name: Extent,
    args: Extent,
}

#[derive(Debug)]
struct Let {
    pattern: Pattern,
    value: Option<Box<Expression>>,
}

#[derive(Debug)]
struct Assign {
    name: Extent,
    value: Box<Expression>,
}

#[derive(Debug)]
struct Tuple {
    members: Vec<Expression>,
}

#[derive(Debug)]
struct FieldAccess {
    value: Box<Expression>,
    field: Extent
}

#[derive(Debug)]
struct Value {
    extent: Extent,
}

#[derive(Debug)]
struct FunctionCall {
    name: Extent,
    args: Vec<Expression>,
}

#[derive(Debug)]
struct MethodCall {
    receiver: Box<Expression>,
    name: Extent,
    turbofish: Option<Extent>,
    args: Vec<Expression>,
}

#[derive(Debug)]
struct Loop {
    body: Box<Block>,
}

#[derive(Debug)]
struct Binary {
    op: Extent,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
}

#[derive(Debug)]
struct If {
    condition: Box<Expression>,
    body: Box<Block>,
}

#[derive(Debug)]
struct Match {
    head: Box<Expression>,
    arms: Vec<MatchArm>,
}

#[derive(Debug)]
struct MatchArm {
    pattern: Pattern,
    body: Expression,
}

#[derive(Debug)]
enum ExpressionTail {
    Binary { op: Extent, rhs: Box<Expression> },
    FieldAccess { field: Extent },
    MethodCall { name: Extent, turbofish: Option<Extent>, args: Vec<Expression> },
}

#[derive(Debug)]
enum Pattern {
    Ident { extent: Extent, ident: Extent, tuple: Vec<Pattern> },
    Tuple { extent: Extent, members: Vec<Pattern> },
}

impl Pattern {
    #[allow(dead_code)]
    fn extent(&self) -> Extent {
        use Pattern::*;
        match *self {
            Ident { extent, .. } | Tuple { extent, .. } => extent
        }
    }
}

#[derive(Debug)]
struct Trait {
    extent: Extent,
    name: Extent,
}

#[derive(Debug)]
struct Impl {
    extent: Extent,
    trait_name: Type,
    type_name: Type,
    body: Vec<ImplFunction>,
}

#[derive(Debug)]
struct ImplFunction {
    extent: Extent,
    header: FunctionHeader,
    body: Option<Block>,
}

#[derive(Debug)]
struct Crate {
    extent: Extent,
    name: Extent,
}

#[derive(Debug)]
struct TypeAlias {
    extent: Extent,
    name: Type,
    defn: Type,
}

// TODO: extract to peresil?
fn parse_until<'s, P>(pt: Point<'s>, p: P) -> (Point<'s>, Extent)
    where P: std::str::pattern::Pattern<'s>
{
    let end = pt.s.find(p).unwrap_or(pt.s.len());
    let k = &pt.s[end..];
    (Point { s: k, offset: pt.offset + end }, (pt.offset, pt.offset + end))
}

// TODO: extract to peresil?
fn parse_nested_until<'s>(open: char, close: char) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, Extent> {
    move |_, pt| {
        let mut depth: usize = 0;
        let spt = pt;

        let val = |len| {
            let pt = Point { s: &pt.s[len..], offset: pt.offset + len };
            Progress::success(pt, ex(spt, pt))
        };

        for (i, c) in pt.s.char_indices() {
            if c == close && depth == 0 {
                return val(i);
            } else if c == close {
                depth -= 1;
            } else if c == open {
                depth += 1;
            }
        }
        val(pt.s.len())
    }
}

// TODO: extract to peresil
fn one_or_more<'s, F, T>(pm: &mut Master<'s>, pt: Point<'s>, mut f: F) -> Progress<'s, Vec<T>>
    where F: FnMut(&mut Master<'s>, Point<'s>) -> Progress<'s, T>,
{
    let (pt, head) = try_parse!(f(pm, pt));
    let (pt, mut tail) = try_parse!(pm.zero_or_more(pt, f));

    tail.insert(0, head);
    Progress::success(pt, tail)
}

// TODO: extract to peresil
fn one_or_more2<'s, F, T>(f: F) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, Vec<T>>
    where F: Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, T>,
{
    move |pm, pt| {
        let (pt, head) = try_parse!(f(pm, pt));
        let (pt, mut tail) = try_parse!(pm.zero_or_more(pt, &f));// what why ref

        tail.insert(0, head);
        Progress::success(pt, tail)
    }
}

// TODO: extract to peresil
macro_rules! sequence {
    ($pm:expr, $pt:expr, {$x:ident = $parser:expr; $($rest:tt)*}, $creator:expr) => {{
        let (pt, $x) = try_parse!($parser($pm, $pt));
        sequence!($pm, pt, {$($rest)*}, $creator)
    }};
    ($pm:expr, $pt:expr, {}, $creator:expr) => {
        Progress::success($pt, $creator($pm, $pt))
    };
}

// TODO: promote?
fn comma_tail<'s, F, T>(f: F) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, T>
    where F: Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, T>
{
    move |pm, pt| {
        sequence!(pm, pt, {
            v  = f;
            _x = optional(whitespace);
            _x = optional(literal(","));
            _x = optional(whitespace);
        }, |_, _| v)
    }
}

// TODO: promote?
fn optional<'s, F, T>(f: F) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, Option<T>>
    where F: Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, T>
{
    move |pm, pt| pm.optional(pt, &f) // what why ref?
}

// TODO: promote?
fn zero_or_more<'s, F, T>(f: F) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, Vec<T>>
    where F: Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, T>
{
    move |pm, pt| pm.zero_or_more(pt, &f) // what why ref?
}

#[allow(dead_code)]
fn map<'s, P, F, T, U>(p: P, f: F) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, U>
    where P: Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, T>,
          F: Fn(T) -> U
{
    move |pm, pt| {
        p(pm, pt).map(&f)
    }
}

// todo: promote?
#[allow(dead_code)]
fn inspect<'s, F>(f: F) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, ()>
    where F: Fn(Point<'s>)
{
    move |_, pt| {
        f(pt);
        Progress::success(pt, ())
    }
}

// TODO: can we transofrm this to (pm, pt)?
fn top_level<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    pm.alternate(pt)
        .one(comment)
        .one(function)
        .one(p_enum)
        .one(p_trait)
        .one(p_impl)
        .one(attribute)
        .one(extern_crate)
        .one(p_use)
        .one(type_alias)
        .one(whitespace)
        .finish()
}

fn literal<'s>(expected: &'static str) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, &'s str> {
    move |_pm, pt| pt.consume_literal(expected).map_err(|_| Error::Literal(expected))
}

fn comment<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    let (pt, _) = try_parse!(literal("//")(pm, pt));
    let spt = pt;
    let (pt, _) = parse_until(pt, "\n");

    Progress::success(pt, TopLevel::Comment(ex(spt, pt)))
}

fn function<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    let spt          = pt;
    sequence!(pm, pt, {
        header = function_header;
        _x     = optional(whitespace);
        body   = block;
    }, |_, pt| TopLevel::Function(Function {
        extent: ex(spt, pt),
        header,
        body,
    }))
}

fn ext<'s, F, T>(f: F) -> impl Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, Extent>
    where F: Fn(&mut Master<'s>, Point<'s>) -> Progress<'s, T>
{
    move |pm, pt| {
        let spt = pt;
        let (pt, _) = try_parse!(f(pm, pt));
        Progress::success(pt, ex(spt, pt))
    }
}

fn function_header<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, FunctionHeader> {
    let spt = pt;
    sequence!(pm, pt, {
        visibility  = optional(ext(literal("pub")));
        _x          = optional(whitespace);
        _x          = literal("fn");
        _x          = optional(whitespace);
        name        = ident;
        generics    = optional(function_generic_declarations);
        arguments   = function_arglist;
        _x          = optional(whitespace);
        return_type = optional(function_return_type);
        _x          = optional(whitespace);
        wheres      = optional(function_where_clause);
    }, |_, pt| FunctionHeader {
        extent: ex(spt, pt),
        visibility,
        name,
        generics: generics.unwrap_or_else(Vec::new),
        arguments,
        return_type,
        wheres: wheres.unwrap_or_else(Vec::new),
    })
}

fn ident<'s>(_pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    let spt = pt;
    let (pt, ex) = parse_until(pt, |c| {
        ['!', '(', ')', ' ', '<', '>', '{', '}', ':', ',', ';', '/', '.'].contains(&c)
    });
    if pt.offset <= spt.offset {
        Progress::failure(pt, Error::IdentNotFound)
    } else {
        Progress::success(pt, ex)
    }
}

fn function_generic_declarations<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Vec<Generic>> {
    let (pt, _)     = try_parse!(literal("<")(pm, pt));
    let (pt, decls) = try_parse!(one_or_more(pm, pt, generic_declaration));
    let (pt, _)     = try_parse!(literal(">")(pm, pt));

    Progress::success(pt, decls)
}

fn generic_declaration<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Generic> {
    ident(pm, pt)
}

fn function_arglist<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Vec<Argument>> {
    let (pt, _)        = try_parse!(literal("(")(pm, pt));
    let (pt, self_arg) = try_parse!(optional(self_argument)(pm, pt));
    let (pt, mut args) = try_parse!(zero_or_more(function_argument)(pm, pt));
    let (pt, _)        = try_parse!(literal(")")(pm, pt));

    if let Some(arg) = self_arg {
        args.insert(0, arg);
    }
    Progress::success(pt, args)
}

fn self_argument<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Argument> {
    let (pt, _) = try_parse!(optional(literal("&"))(pm, pt));
    let (pt, _) = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _) = try_parse!(optional(literal("mut"))(pm, pt));
    let (pt, _) = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _) = try_parse!(literal("self")(pm, pt));
    let (pt, _) = try_parse!(optional(literal(","))(pm, pt));

    Progress::success(pt, Argument::SelfArgument)
}

fn function_argument<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Argument> {
    let (pt, name) = try_parse!(ident(pm, pt));
    let (pt, _)    = try_parse!(literal(":")(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, typ)  = try_parse!(ident(pm, pt));
    let (pt, _)    = try_parse!(optional(literal(","))(pm, pt));

    Progress::success(pt, Argument::Named { name, typ })
}

fn function_return_type<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Type> {
    let (pt, _) = try_parse!(literal("->")(pm, pt));
    let (pt, _) = try_parse!(optional(whitespace)(pm, pt));
    let (pt, t) = try_parse!(typ(pm, pt));

    Progress::success(pt, t)
}

fn function_where_clause<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Vec<Where>> {
    let (pt, _) = try_parse!(literal("where")(pm, pt));
    let (pt, _) = try_parse!(whitespace(pm, pt));

    one_or_more(pm, pt, function_where)
}

fn function_where<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Where> {
    let (pt, name)   = try_parse!(ident(pm, pt));
    let (pt, _)      = try_parse!(literal(":")(pm, pt));
    let (pt, _)      = try_parse!(optional(whitespace)(pm, pt));
    let (pt, bounds) = try_parse!(ident(pm, pt));
    let (pt, _)      = try_parse!(optional(literal(","))(pm, pt));

    Progress::success(pt, Where { name, bounds })
}

fn block<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Block> {
    let spt = pt;
    sequence!(pm, pt, {
        _x    = literal("{");
        _x    = optional(whitespace);
        stmts = zero_or_more(statement);
        expr  = optional(expression);
        _x    = optional(whitespace);
        _x    = literal("}");
    }, |_, pt| {
        let mut stmts = stmts;
        let mut expr = expr;

        if expr.is_none() && stmts.last().map_or(false, Statement::is_implicit) {
            expr = stmts.pop().and_then(Statement::implicit);
        }

        Block {
            extent: ex(spt, pt),
            statements: stmts,
            expression: expr,
        }
    })
}

fn statement<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Statement> {
    sequence!(pm, pt, {
        _x   = optional(whitespace);
        expr = statement_inner;
        _x   = optional(whitespace);
    }, |_, _| expr)
}

fn statement_inner<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Statement> {
    pm.alternate(pt)
        .one(explicit_statement)
        .one(implicit_statement)
        .finish()
}

fn explicit_statement<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Statement> {
    sequence!(pm, pt, {
        expr = expression;
        _x = literal(";");
    }, |_, _| Statement::Explicit(expr))
}

// idea: trait w/associated types to avoid redefin fn types?

fn implicit_statement<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Statement> {
    let spt = pt;
    let (pt, kind) = try_parse!(expression_ending_in_brace(pm, pt));

    Progress::success(pt, Statement::Implicit(Expression { extent: ex(spt, pt), kind: kind }))
}

fn expression_ending_in_brace<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, ExpressionKind> {
    pm.alternate(pt)
        .one(map(expr_if, ExpressionKind::If))
        .one(map(expr_loop, ExpressionKind::Loop))
        .one(map(expr_match, ExpressionKind::Match))
        .one(map(expr_block, ExpressionKind::Block))
        .finish()
}

fn expression<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Expression> {
    let spt        = pt;
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, kind) = try_parse!({
        pm.alternate(pt)
            .one(expression_ending_in_brace)
            .one(map(expr_macro_call, ExpressionKind::MacroCall))
            .one(map(expr_let, ExpressionKind::Let))
            .one(map(expr_assign, ExpressionKind::Assign))
            .one(map(expr_function_call, ExpressionKind::FunctionCall))
            .one(map(expr_tuple, ExpressionKind::Tuple))
            .one(expr_true)
            .one(map(expr_value, ExpressionKind::Value))
            .finish()
    });
    let mpt = pt;


//    let (pt, tail) = try_parse!(optional(expression_tail)(pm, pt));
//    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));

    let mut expression = Expression {
        extent: ex(spt, mpt),
        kind,
    };

    let mut pt = pt;
    loop {
        let (pt2, tail) = try_parse!(optional(expression_tail)(pm, pt));
        pt = pt2;
        match tail {
            Some(ExpressionTail::Binary { op, rhs }) => {
                expression = Expression {
                    extent: ex(spt, pt),
                    kind: ExpressionKind::Binary(Binary {
                        op: op,
                        lhs: Box::new(expression),
                        rhs: rhs,
                    })
                }
            }
            Some(ExpressionTail::FieldAccess { field }) => {
                //mid.insert(0, expression);
                expression = Expression {
                    extent: ex(spt, pt),
                    kind: ExpressionKind::FieldAccess(FieldAccess {
                        value: Box::new(expression),
                        field: field,
                    })
                }
            }
            Some(ExpressionTail::MethodCall { name, turbofish, args }) => {
                expression = Expression {
                    extent: ex(spt, pt),
                    kind: ExpressionKind::MethodCall(MethodCall {
                        receiver: Box::new(expression),
                        name: name,
                        turbofish: turbofish,
                        args: args
                    })
                }
            }
            None => break,
        }
    }

    Progress::success(pt, expression)
}

fn expr_macro_call<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, MacroCall> {
    sequence!(pm, pt, {
        name = ident;
        _x   = literal("!");
        _x   = literal("(");
        args = parse_nested_until('(', ')');
        _x   = literal(")");
    }, |_, _| MacroCall { name, args })
}

fn expr_let<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Let> {
    let (pt, _)       = try_parse!(literal("let")(pm, pt));
    let (pt, _)       = try_parse!(whitespace(pm, pt));
    let (pt, pattern) = try_parse!(pattern(pm, pt));
    let (pt, _)       = try_parse!(optional(whitespace)(pm, pt));
    let (pt, value)   = try_parse!(optional(expr_let_rhs)(pm, pt));

    Progress::success(pt, Let {
        pattern,
        value: value.map(Box::new),
    })
}

fn expr_let_rhs<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Expression> {
    let (pt, _)     = try_parse!(literal("=")(pm, pt));
    let (pt, _)     = try_parse!(optional(whitespace)(pm, pt));
    let (pt, value) = try_parse!(expression(pm, pt));

    Progress::success(pt, value)
}

fn expr_assign<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Assign> {
    sequence!(pm, pt, {
        name  = ident;
        _x    = optional(whitespace);
        _x    = literal("=");
        _x    = optional(whitespace);
        value = expression;
    }, |_, _| Assign { name, value: Box::new(value) })
}

fn expr_if<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, If> {
    sequence!(pm, pt, {
        _x        = literal("if");
        _x        = whitespace;
        condition = expression;
        _x        = optional(whitespace);
        body      = block;
    }, |_, _| If { condition: Box::new(condition), body: Box::new(body) })
}

fn expr_loop<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Loop> {
    let (pt, _)    = try_parse!(literal("loop")(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, body) = try_parse!(block(pm, pt));

    Progress::success(pt, Loop {
        body: Box::new(body),
    })
}

fn expr_match<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Match> {
    let (pt, _)    = try_parse!(literal("match")(pm, pt));
    let (pt, _)    = try_parse!(whitespace(pm, pt));
    let (pt, head) = try_parse!(expression(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)    = try_parse!(literal("{")(pm, pt));
    let (pt, arms) = try_parse!(zero_or_more(match_arm)(pm, pt));
    let (pt, _)    = try_parse!(literal("}")(pm, pt));

    Progress::success(pt, Match {
        head: Box::new(head),
        arms,
    })
}

fn match_arm<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, MatchArm> {
    let (pt, _)       = try_parse!(optional(whitespace)(pm, pt));
    let (pt, pattern) = try_parse!(pattern(pm, pt));
    let (pt, _)       = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)       = try_parse!(literal("=>")(pm, pt));
    let (pt, _)       = try_parse!(optional(whitespace)(pm, pt));
    let (pt, body)    = try_parse!(expression(pm, pt));
    let (pt, _)       = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)       = try_parse!(optional(literal(","))(pm, pt));
    let (pt, _)       = try_parse!(optional(whitespace)(pm, pt));

    Progress::success(pt, MatchArm {
        pattern, body
    })
}

fn expr_tuple<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Tuple> {
    let (pt, _) = try_parse!(literal("(")(pm, pt));
    let (pt, v) = try_parse!(zero_or_more(comma_tail(expression))(pm, pt));
    let (pt, _) = try_parse!(literal(")")(pm, pt));

    Progress::success(pt, Tuple {
        members: v,
    })
}

fn expr_block<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Box<Block>> {
    block(pm, pt).map(Box::new)
}

fn expr_value<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Value> {
    pathed_ident(pm, pt).map(|extent| Value { extent })
}

fn expr_function_call<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, FunctionCall> {
    sequence!(pm, pt, {
        name = pathed_ident;
        _x   = literal("(");
        args = zero_or_more(comma_tail(expression));
        _x   = literal(")");
    }, |_, _| FunctionCall { name, args })
}

fn expr_true<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, ExpressionKind> {
    let (pt, _) = try_parse!(literal("true")(pm, pt));

    Progress::success(pt, ExpressionKind::True)
}

fn expression_tail<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, ExpressionTail> {
    pm.alternate(pt)
        .one(expr_tail_binary)
        .one(expr_tail_method_call)
        .one(expr_tail_field_access)
        .finish()
}

fn expr_tail_binary<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, ExpressionTail> {
    sequence!(pm, pt, {
        _x  = optional(whitespace);
        op  = binary_op;
        _x  = optional(whitespace);
        rhs = expression;
    }, |_, _| ExpressionTail::Binary { op, rhs: Box::new(rhs) })
}

fn binary_op<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    // Two characters before one to avoid matching += as +
    pm.alternate(pt)
        .one(ext(literal("+=")))
        .one(ext(literal("-=")))
        .one(ext(literal("*=")))
        .one(ext(literal("/=")))
        .one(ext(literal("%=")))
        .one(ext(literal("<=")))
        .one(ext(literal(">=")))
        .one(ext(literal("+")))
        .one(ext(literal("-")))
        .one(ext(literal("*")))
        .one(ext(literal("/")))
        .one(ext(literal("%")))
        .one(ext(literal("<")))
        .one(ext(literal(">")))
        .finish()
}

fn expr_tail_method_call<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, ExpressionTail> {
    sequence!(pm, pt, {
        _x        = literal(".");
        name      = ident;
        turbofish = optional(turbofish);
        _x        = literal("(");
        args      = zero_or_more(comma_tail(expression));
        _x        = literal(")");
    }, |_, _| ExpressionTail::MethodCall { name, turbofish, args })
}

fn expr_tail_field_access<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, ExpressionTail> {
    sequence!(pm, pt, {
        _x = literal(".");
        field = ident;
    }, |_, _| ExpressionTail::FieldAccess { field })
}

fn pathed_ident<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    let spt = pt;
    sequence!(pm, pt, {
        _x = ident;
        _x = zero_or_more(path_component);
        _x = optional(turbofish);
    }, |_, pt| ex(spt, pt))
}

fn path_component<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    let spt = pt;
    sequence!(pm, pt, {
        _x = literal("::");
        _x = ident;
    }, |_, pt| ex(spt, pt))
}

fn turbofish<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    sequence!(pm, pt, {
        _x    = literal("::<");
        types = ext(one_or_more2(comma_tail(typ)));
        _x    = literal(">");
    }, |_, _| types)
}

fn pattern<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Pattern> {
    pm.alternate(pt)
        .one(pattern_ident)
        .one(pattern_tuple)
        .finish()
}

fn pattern_ident<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Pattern> {
    let spt = pt;
    sequence!(pm, pt, {
        _x    = optional(literal("mut"));
        _x    = optional(whitespace);
        ident = pathed_ident;
        tuple = optional(pattern_tuple_inner);
    }, |_, pt| Pattern::Ident { extent: ex(spt, pt), ident, tuple: tuple.unwrap_or_else(Vec::new) })
}

fn pattern_tuple<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Pattern> {
    let spt = pt;
    let (pt, members) = try_parse!(pattern_tuple_inner(pm, pt));
    Progress::success(pt, Pattern::Tuple { extent: ex(spt, pt), members })
}

fn pattern_tuple_inner<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Vec<Pattern>> {
    sequence!(pm, pt, {
        _x           = literal("(");
        sub_patterns = zero_or_more(comma_tail(pattern));
        _x           = literal(")");
    }, |_, _| sub_patterns)
}

fn p_enum<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    p_enum_inner(pm, pt).map(TopLevel::Enum)
}

fn p_enum_inner<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Enum> {
    let spt = pt;
    sequence!(pm, pt, {
        _x       = literal("enum");
        _x       = whitespace;
        name     = ident;
        _x       = optional(whitespace);
        _x       = literal("{");
        _x       = optional(whitespace);
        variants = zero_or_more(comma_tail(enum_variant));
        _x       = optional(whitespace);
        _x       = literal("}");
    }, |_, pt| Enum {
        extent: ex(spt, pt),
        name,
        variants,
    })
}

fn enum_variant<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, EnumVariant> {
    let spt        = pt;
    let (pt, name) = try_parse!(ident(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, body) = try_parse!(optional(enum_variant_body)(pm, pt));

    Progress::success(pt,  EnumVariant {
        extent: ex(spt, pt),
        name,
        body: body.unwrap_or_else(Vec::new),
    })
}

fn enum_variant_body<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Vec<EnumVariantBody>> {
    let (pt, _)     = try_parse!(literal("(")(pm, pt));
    let (pt, types) = try_parse!(zero_or_more(comma_tail(typ))(pm, pt));
    let (pt, _)     = try_parse!(literal(")")(pm, pt));

    Progress::success(pt, types)
}

fn p_trait<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    let spt        = pt;
    let (pt, _)    = try_parse!(literal("trait")(pm, pt));
    let (pt, _)    = try_parse!(whitespace(pm, pt));
    let (pt, name) = try_parse!(ident(pm, pt));
    let (pt, _)    = try_parse!(whitespace(pm, pt));
    let (pt, _)    = try_parse!(literal("{}")(pm, pt));

    Progress::success(pt, TopLevel::Trait(Trait {
        extent: ex(spt, pt),
        name,
    }))
}

fn p_impl<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    p_impl_inner(pm, pt).map(TopLevel::Impl)
}

fn p_impl_inner<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Impl> {
    let spt              = pt;
    let (pt, _)          = try_parse!(literal("impl")(pm, pt));
    let (pt, _)          = try_parse!(whitespace(pm, pt));
    let (pt, trait_name) = try_parse!(typ(pm, pt));
    let (pt, _)          = try_parse!(whitespace(pm, pt));
    let (pt, _)          = try_parse!(literal("for")(pm, pt));
    let (pt, _)          = try_parse!(whitespace(pm, pt));
    let (pt, type_name)  = try_parse!(typ(pm, pt));
    let (pt, _)          = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)          = try_parse!(literal("{")(pm, pt));
    let (pt, _)          = try_parse!(optional(whitespace)(pm, pt));
    let (pt, body)       = try_parse!(zero_or_more(impl_function)(pm, pt));
    let (pt, _)          = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)          = try_parse!(literal("}")(pm, pt));

    Progress::success(pt, Impl {
        extent: ex(spt, pt),
        trait_name,
        type_name,
        body,
    })
}

fn impl_function<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, ImplFunction> {
    let spt = pt;
    let (pt, header) = try_parse!(function_header(pm, pt));
    let (pt, body)   = try_parse!(optional(block)(pm, pt));

    Progress::success(pt, ImplFunction {
        extent: ex(spt, pt),
        header,
        body,
    })
}

// TODO: optional could take E that is `into`, or just a different one

fn attribute<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    let spt = pt;
    let (pt, _) = try_parse!(literal("#")(pm, pt));
    let (pt, _) = try_parse!(optional(literal("!"))(pm, pt));
    let (pt, _) = try_parse!(literal("[")(pm, pt));
    let (pt, _) = parse_until(pt, "]");
    let (pt, _) = try_parse!(literal("]")(pm, pt));

    Progress::success(pt, TopLevel::Attribute(ex(spt, pt)))
}

fn extern_crate<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    let spt = pt;
    let (pt, _)    = try_parse!(literal("extern")(pm, pt));
    let (pt, _)    = try_parse!(whitespace(pm, pt));
    let (pt, _)    = try_parse!(literal("crate")(pm, pt));
    let (pt, _)    = try_parse!(whitespace(pm, pt));
    let (pt, name) = try_parse!(ident(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)    = try_parse!(literal(";")(pm, pt));

    Progress::success(pt, TopLevel::ExternCrate(Crate {
        extent: ex(spt, pt),
        name,
    }))
}

fn p_use<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    p_use_inner(pm, pt).map(TopLevel::Use)
}

fn p_use_inner<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Use> {
    let spt = pt;
    sequence!(pm, pt, {
        _x   = literal("use");
        _x   = whitespace;
        name = pathed_ident;
        _x   = literal(";");
    }, |_, pt| Use {
        extent: ex(spt, pt),
        name
    })
}

fn type_alias<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    let spt = pt;
    let (pt, _)    = try_parse!(literal("type")(pm, pt));
    let (pt, _)    = try_parse!(whitespace(pm, pt));
    let (pt, name) = try_parse!(typ(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)    = try_parse!(literal("=")(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, defn) = try_parse!(typ(pm, pt));
    let (pt, _)    = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _)    = try_parse!(literal(";")(pm, pt));

    Progress::success(pt, TopLevel::TypeAlias(TypeAlias {
        extent: ex(spt, pt),
        name,
        defn,
    }))
}

fn typ<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    let spt = pt;
    let (pt, _) = try_parse!(optional(literal("&"))(pm, pt));
    let (pt, _) = try_parse!(optional(lifetime)(pm, pt));
    let (pt, _) = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _) = try_parse!(pathed_ident(pm, pt));
    let (pt, _) = try_parse!(optional(typ_generics)(pm, pt));

    Progress::success(pt, ex(spt, pt))
}

fn typ_generics<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    let spt = pt;
    let (pt, _) = try_parse!(literal("<")(pm, pt));
    let (pt, _) = try_parse!(optional(whitespace)(pm, pt));
    let (pt, _) = try_parse!(optional(typ_generic_lifetimes)(pm, pt));
    let (pt, _) = try_parse!(optional(type_generic_types)(pm, pt));
    let (pt, _) = try_parse!(literal(">")(pm, pt));

    Progress::success(pt, ex(spt, pt))
}

fn typ_generic_lifetimes<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Vec<Extent>> {
    one_or_more(pm, pt, comma_tail(lifetime))
}

fn lifetime<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Extent> {
    let (pt, _) = try_parse!(literal("'")(pm, pt));
    let (pt, _) = try_parse!(optional(whitespace)(pm, pt));
    ident(pm, pt)
}

fn type_generic_types<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, Vec<Extent>> {
    one_or_more(pm, pt, comma_tail(typ))
}

fn whitespace<'s>(pm: &mut Master<'s>, pt: Point<'s>) -> Progress<'s, TopLevel> {
    let spt = pt;

    let (pt, _) = try_parse!(one_or_more(pm, pt, |pm, pt| {
        pm.alternate(pt)
            .one(literal(" "))
            .one(literal("\t"))
            .one(literal("\r"))
            .one(literal("\n"))
            .finish()
    }));

    Progress::success(pt, TopLevel::Whitespace(ex(spt, pt)))
}

#[cfg(test)]
mod test {
    use super::*;

    fn qp<'s, F, T>(f: F, s: &'s str) -> peresil::Progress<Point<'s>, T, Vec<Error>>
        where F: FnOnce(&mut Master<'s>, Point<'s>) -> Progress<'s, T>
    {
        // TODO: Master::once()?
        let mut pm = Master::new();
        let pt = Point::new(s);
        let r = f(&mut pm, pt);
        pm.finish(r)
    }

    #[test]
    fn top_level_use() {
        let p = qp(p_use_inner, "use foo::Bar;");
        assert_eq!(unwrap_progress(p).extent, (0, 13))
    }

    #[test]
    fn enum_with_trailing_stuff() {
        let p = qp(p_enum_inner, "enum A {} impl Foo for Bar {}");
        assert_eq!(unwrap_progress(p).extent, (0, 9))
    }

    #[test]
    fn fn_with_public_modifier() {
        let p = qp(function_header, "pub fn foo()");
        assert_eq!(unwrap_progress(p).extent, (0, 12))
    }

    #[test]
    fn fn_with_self_type() {
        let p = qp(function_header, "fn foo(&self)");
        assert_eq!(unwrap_progress(p).extent, (0, 13))
    }

    #[test]
    fn fn_with_return_type() {
        let p = qp(function_header, "fn foo() -> bool");
        assert_eq!(unwrap_progress(p).extent, (0, 16))
    }

    #[test]
    fn block_promotes_implicit_statement_to_expression() {
        let p = qp(block, "{ if a {} }");
        let p = unwrap_progress(p);
        assert!(p.statements.is_empty());
        assert_eq!(p.expression.unwrap().extent, (2, 9));
    }

    #[test]
    fn statement_match_no_semicolon() {
        let p = qp(statement, "match a { _ => () }");
        assert_eq!(unwrap_progress(p).implicit().unwrap().extent, (0, 19))
    }

    #[test]
    fn expr_true() {
        let p = qp(expression, "true");
        assert_eq!(unwrap_progress(p).extent, (0, 4))
    }

    #[test]
    fn expr_let_mut() {
        let p = qp(expression, "let mut pm = Master::new()");
        assert_eq!(unwrap_progress(p).extent, (0, 26))
    }

    #[test]
    fn expr_let_no_value() {
        let p = qp(expression, "let pm");
        assert_eq!(unwrap_progress(p).extent, (0, 6))
    }

    #[test]
    fn expr_assign() {
        let p = qp(expression, "a = b");
        assert_eq!(unwrap_progress(p).extent, (0, 5))
    }

    #[test]
    fn expr_value_with_path() {
        let p = qp(expression, "Master::new()");
        assert_eq!(unwrap_progress(p).extent, (0, 13))
    }

    #[test]
    fn expr_field_access() {
        let p = qp(expression, "foo.bar");
        assert_eq!(unwrap_progress(p).extent, (0, 7))
    }

    #[test]
    fn expr_field_access_multiple() {
        let p = qp(expression, "foo.bar.baz");
        assert_eq!(unwrap_progress(p).extent, (0, 11))
    }

    #[test]
    fn expr_function_call() {
        let p = qp(expression, "foo()");
        assert_eq!(unwrap_progress(p).extent, (0, 5))
    }

    #[test]
    fn pathed_ident_with_turbofish() {
        let p = qp(pathed_ident, "foo::<Vec<u8>>");
        assert_eq!(unwrap_progress(p), (0, 14))
    }

    #[test]
    fn expr_function_call_with_args() {
        let p = qp(expression, "foo(true)");
        assert_eq!(unwrap_progress(p).extent, (0, 9))
    }

    #[test]
    fn expr_method_call() {
        let p = qp(expression, "foo.bar()");
        assert_eq!(unwrap_progress(p).extent, (0, 9))
    }

    #[test]
    fn expr_method_call_multiple() {
        let p = qp(expression, "foo.bar().baz()");
        assert_eq!(unwrap_progress(p).extent, (0, 15))
    }

    #[test]
    fn expr_method_call_with_turbofish() {
        let p = qp(expression, "foo.bar::<u8>()");
        assert_eq!(unwrap_progress(p).extent, (0, 15))
    }

    #[test]
    fn expr_method_call_with_turbofish_2() {
        let p = qp(expression, "e.into_iter().collect::<BTreeSet<_>>()");
        assert_eq!(unwrap_progress(p).extent, (0, 38))
    }

    #[test]
    fn expr_loop() {
        let p = qp(expression, "loop {}");
        assert_eq!(unwrap_progress(p).extent, (0, 7))
    }

    #[test]
    fn expr_match() {
        let p = qp(expression, "match foo { _ => () }");
        assert_eq!(unwrap_progress(p).extent, (0, 21))
    }

    #[test]
    fn expr_tuple() {
        let p = qp(expression, "(1, 2)");
        assert_eq!(unwrap_progress(p).extent, (0, 6))
    }

    #[test]
    fn expr_block() {
        let p = qp(expression, "{}");
        assert_eq!(unwrap_progress(p).extent, (0, 2))
    }

    #[test]
    fn expr_if() {
        let p = qp(expression, "if true {}");
        assert_eq!(unwrap_progress(p).extent, (0, 10))
    }

    #[test]
    fn expr_binary_op() {
        let p = qp(expression, "a < b");
        assert_eq!(unwrap_progress(p).extent, (0, 5))
    }

    #[test]
    fn expr_binary_multiple() {
        let p = qp(expression, "1 + 2 + 3");
        assert_eq!(unwrap_progress(p).extent, (0, 9))
    }

    #[test]
    fn expr_binary_op_two_char() {
        let p = qp(expression, "a >= b");
        assert_eq!(unwrap_progress(p).extent, (0, 6))
    }

    #[test]
    fn expr_braced_true() {
        let p = qp(expression, "{ true }");
        assert_eq!(unwrap_progress(p).extent, (0, 8))
    }

    #[test]
    fn pattern_with_path() {
        let p = qp(pattern, "foo::Bar::Baz");
        assert_eq!(unwrap_progress(p).extent(), (0, 13))
    }

    #[test]
    fn pattern_with_tuple() {
        let p = qp(pattern, "(a, b)");
        assert_eq!(unwrap_progress(p).extent(), (0, 6))
    }

    #[test]
    fn pattern_with_enum_tuple() {
        let p = qp(pattern, "Baz(a)");
        assert_eq!(unwrap_progress(p).extent(), (0, 6))
    }

    #[test]
    fn expr_macro_call_with_nested_parens() {
        let p = qp(expression, "foo!(())");
        assert_eq!(unwrap_progress(p).extent, (0, 8))
    }

    fn unwrap_progress<P, T, E>(p: peresil::Progress<P, T, E>) -> T
        where P: std::fmt::Debug,
              E: std::fmt::Debug,
    {
        match p {
            peresil::Progress { status: peresil::Status::Success(v), .. } => v,
            peresil::Progress { status: peresil::Status::Failure(e), point } => {
                panic!("Failed parsing at {:?}: {:?}", point, e)
            }
        }
    }
}
