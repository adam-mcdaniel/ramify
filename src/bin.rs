#[macro_use]
extern crate maplit;
use ramify::{
    builtin, foreign, parse_ast, parse_data_declaration, parse_expression,
    parse_function_declaration, parse_type_declaration, Combinator, Golang, Lambda, Reduce,
    ReductionError, Target, B, C, I, K, S,
};

use comment::shell::strip;
use nom::IResult;

fn main() {
    // let f = Lambda::lambda(
    //     "a",
    //     Lambda::lambda(
    //         "b",
    //         Lambda::var("b").applied_to(Lambda::Combinator(Combinator::List(vec![
    //             Combinator::Integer(10),
    //             Combinator::String(String::from("wow!")),
    //             Combinator::Table(btreemap! {
    //                 String::from("a") => Combinator::Float(2.5),
    //                 String::from("b") => Combinator::Table(btreemap! {
    //                     String::from("c") => Lambda::lambda("x", Lambda::lambda("y", Lambda::var("y"))).to_combinator(),
    //                     String::from("d") => Lambda::Combinator(foreign("println")).applied_to(Lambda::Combinator(Combinator::String(String::from("Hello world")))).to_combinator()
    //                 }),
    //             }),
    //         ])))
    //     )
    // )
    // .applied_to(Lambda::Combinator(Combinator::Integer(0)))
    // .applied_to(Lambda::Combinator(foreign("println")))
    // .to_combinator()
    // ;
    // println!("{}\n=>\n{}", f.clone(), Golang.compile(f).unwrap());

    // println!("{:?}", parse_expression("a.b.a").unwrap());
    // println!("{:?}", parse_expression("a.b.b").unwrap());
    // println!("{:?}", parse_expression("a.b.a b").unwrap());
    // println!("{:?}", parse_expression("a.b.a (a b)").unwrap());

    // let f = parse_function_declaration(
    //     "let add s1 s2 = let Square(w1, h1) = s1 in
    //                     let Square(w2, h2) = s2 in
    //                         Square(w1+w2, h1+h2)",
    // );
    // println!("{:?}", f);

    // let mut expr = parse_expression("Square(a, b)").unwrap().1;

    // for cons in &data.cons {
    //     expr = (*expr.replace_constructors(cons)).clone();
    // }

    // expr = (*expr.replace_constructors(&square)).clone();
    let input = strip(
        r#"
const True  = a.b.a
const False = a.b.b
const Null  = x.True

const ZERO = False
const ONE  = f.x.f x
const TWO  = f.x.f (f x)
const THREE  = f.x.f (f (f x))
let and a b = a b a
let or  a b = a a b
let not a = a False True
data Result = Ok(x) | Err(e)
let input n = if n then Ok("test") else Err("invalid argument to input")
let test n = case input n of
                | Ok(x) => print "You said: " x
                | Err(e) => print "Error: " e
let succ   n f x =   f (n f x)
let pred   n f x = n (g.h.h (g f)) (u.x) (u.u)
let sub  m n     = n pred m
let add  m n f x = m f (n f x)
let mul  m n f   = m   (n f)
let pow  base exponent = exponent base


type Point(x, y)

let add_points p1 p2 = let Point(x1, y1) = p1 in
                            let Point(x2, y2) = p2 in Point(x1, y2)

data Shape = Rectangle(width, height) | Circle(radius)
let test_shape shape = case shape of
                            | Rectangle(w, h) => TWO
                            | Circle(r) => ONE

# let main _ = let Point(x, y) = add_points Point(1, 1) Point(2, 2) in x

let factorial n = if n>1 then n*(rec n-1) else 1

let main _ = factorial 5
    "#,
    )
    .unwrap();
    let ast = parse_ast(&input);
    println!("{:#?}", ast);
    let ast = ast.unwrap().1;

    // for f in ast.get_functions() {
    //     println!(
    //         "let {} = {:?} ~ ({})",
    //         f.get_name(),
    //         f.get_body(),
    //         f.get_body().number_of_arguments()
    //     );
    // }

    // for c in ast.get_constants() {
    //     println!(
    //         "const {} = {:?} ~ ({})",
    //         c.get_name(),
    //         c.get_body(),
    //         c.get_body().number_of_arguments()
    //     );
    // }

    let result = ast.reduce(&()).unwrap().to_combinator();
    println!("let main = {}", result);
    println!("\n\nGOLANG => \n{}", Golang.compile(result).unwrap());

    // for f in ast.get_functions() {
    //     println!(
    //         "{} = {}",
    //         f.get_name(),
    //         f.reduce(&()).unwrap()
    //     );
    // }
}
