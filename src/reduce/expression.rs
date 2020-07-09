use crate::{
    ast::{Constructor, Data, Expression},
    builtin, foreign,
    lambda::Lambda,
    reduce::{Reduce, ReductionError},
    Combinator, Nil, K, S,
};

use alloc::{collections::BTreeMap, vec::Vec};

fn y(f: Combinator) -> Combinator {
    builtin("Y", move |x| {
        loop {
            let result = f.applied_to(x.clone());
            // println!("result {:?}", result);
            result.applied_to(builtin("Y", y).applied_to(result.clone()));
        }
    })
}

impl Reduce<Vec<Data>> for Expression {
    fn reduce(&self, d: &Vec<Data>) -> Result<Lambda, ReductionError> {
        Ok(match self {
            Self::Greater(a, b) => Lambda::Combinator(builtin("greater", move |c| {
                builtin("greater", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) if m > n => K,
                    _ => S.applied_to(K),
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::Less(a, b) => Lambda::Combinator(builtin("less", move |c| {
                builtin("less", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) if m < n => K,
                    _ => S.applied_to(K),
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::LessEqual(a, b) => Lambda::Combinator(builtin("lesseq", move |c| {
                builtin("lesseq", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) if m <= n => K,
                    _ => S.applied_to(K),
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::GreaterEqual(a, b) => Lambda::Combinator(builtin("greatereq", move |c| {
                builtin("greatereq", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) if m >= n => K,
                    _ => S.applied_to(K),
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::Add(a, b) => Lambda::Combinator(builtin("add", move |c| {
                builtin("add", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) => Combinator::Number(m + n),
                    (Combinator::String(m), Combinator::String(n)) => {
                        Combinator::String(m.clone() + &n)
                    }
                    _ => Nil,
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::Subtract(a, b) => Lambda::Combinator(builtin("sub", move |c| {
                builtin("sub", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) => Combinator::Number(m - n),
                    _ => Nil,
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::Multiply(a, b) => Lambda::Combinator(builtin("mul", move |c| {
                builtin("mul", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) => Combinator::Number(m * n),
                    _ => Nil,
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::Divide(a, b) => Lambda::Combinator(builtin("div", move |c| {
                builtin("div", move |d| match (c.clone(), d) {
                    (Combinator::Number(m), Combinator::Number(n)) => Combinator::Number(m / n),
                    _ => Nil,
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::TailCall(args) => {
                let mut result = Lambda::var(Expression::RECURSION_ARGUMENT);
                for arg in args.clone() {
                    result = result.applied_to(arg.reduce(d)?);
                }
                result
            }
            Self::And(a, b) => {
                let a = a.reduce(d)?;
                let b = b.reduce(d)?;
                a.clone().applied_to(b).applied_to(a)
            }
            Self::Or(a, b) => {
                let a = a.reduce(d)?;
                let b = b.reduce(d)?;
                a.clone().applied_to(a).applied_to(b)
            }
            Self::Not(a) => {
                let a = a.reduce(d)?;
                let t = Lambda::lambda("a", Lambda::lambda("b", Lambda::var("a")));
                let f = Lambda::lambda("a", Lambda::lambda("b", Lambda::var("b")));
                a.applied_to(f).applied_to(t)
            }
            Self::Equal(a, b) => Lambda::Combinator(builtin("eq", |a| {
                builtin(format!("eq({:?})", a), move |b| {
                    if a == b {
                        Lambda::lambda("a", Lambda::lambda("b", Lambda::var("a"))).to_combinator()
                    } else {
                        Lambda::lambda("a", Lambda::lambda("b", Lambda::var("b"))).to_combinator()
                    }
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::NotEqual(a, b) => Lambda::Combinator(builtin("eq", |a| {
                builtin(format!("eq({:?})", a), move |b| {
                    if a == b {
                        Lambda::lambda("a", Lambda::lambda("b", Lambda::var("a"))).to_combinator()
                    } else {
                        Lambda::lambda("a", Lambda::lambda("b", Lambda::var("b"))).to_combinator()
                    }
                })
            }))
            .applied_to(a.reduce(d)?)
            .applied_to(b.reduce(d)?),
            Self::Negate(a) => Lambda::Combinator(builtin("-", |val| {
                if let Combinator::Number(n) = val {
                    Combinator::Number(-n)
                } else {
                    val.clone()
                }
            }))
            .applied_to(a.reduce(d)?),
            Self::IfThenElse {
                condition,
                then_case,
                else_case,
            } => {
                if then_case.number_of_arguments() != else_case.number_of_arguments() {
                    println!(
                        "Warning: Different type signatures for branches of if expression \n\"{:?}\"",
                        self
                    );
                }

                condition
                    .reduce(d)?
                    .applied_to(then_case.reduce(d)?)
                    .applied_to(else_case.reduce(d)?)
            }
            Self::CaseOf {
                data_type,
                value,
                cases,
                ..
            } => {
                let data = data_type.clone().unwrap();
                let mut result = data.reduce(&())?;
                let mut cases = cases.clone();
                cases.sort_by(
                    |(left_name, left_members, _), (right_name, right_members, _)| {
                        Constructor::new(left_name.clone(), left_members.clone())
                            .cmp(&Constructor::new(right_name.clone(), right_members.clone()))
                    },
                );
                result = result.applied_to(value.reduce(d)?);

                let first_signature = cases[0].2.number_of_arguments();
                for (_, mut members, body) in cases {
                    let mut case_lambda = body.reduce(d)?;

                    if body.number_of_arguments() != first_signature {
                        println!(
                            "Warning: Different type signatures for cases in expression \n\"{:?}\"",
                            self
                        );
                    }

                    members.reverse();
                    for member in members {
                        case_lambda = Lambda::lambda(member, case_lambda)
                    }
                    result = result.applied_to(case_lambda);
                }

                result
            }
            Self::Construct {
                data_type, members, ..
            } => {
                let data_type = data_type.clone().unwrap();
                let mut cons = data_type.reduce(&())?;
                for enumeration in d {
                    if enumeration.get_constructors().contains(&data_type) {
                        cons = data_type.reduce(enumeration)?;
                    }
                }

                let mut result = cons;
                for member in members {
                    result = result.applied_to(member.reduce(d)?);
                }
                result
            }
            Self::Deconstruct {
                members,
                value,
                body,
                ..
            } => {
                let cons = value.reduce(d)?;
                let mut result = body.reduce(d)?;
                let mut members = members.clone();
                members.reverse();
                for member in members {
                    result = Lambda::lambda(member, result);
                }
                cons.applied_to(result)
            }
            Self::Identifier(i) => match i.as_str() {
                "print" => Lambda::Combinator(foreign("print")),
                "println" => Lambda::Combinator(foreign("println")),
                "Y" => Lambda::Combinator(builtin("Y", y)),
                // "Y" => Lambda::Combinator(builtin("Y", move |f| {
                //     builtin("Y", move |x| {
                //         loop {
                //             let result = f.applied_to(x);
                //             result
                //         }
                //     }))
                // }),
                other => Lambda::var(other),
            },
            Self::Application(left, right) => left.reduce(d)?.applied_to(right.reduce(d)?),
            Self::Lambda(arg, body) => Lambda::lambda(arg, body.reduce(d)?),
            Self::Number(n) => Lambda::Combinator(Combinator::Number(*n)),
            Self::String(s) => Lambda::Combinator(Combinator::String(s.clone())),
            Self::List(list) => {
                let mut result = Vec::new();
                for item in list {
                    result.push(item.reduce(d)?.to_combinator());
                }
                Lambda::Combinator(Combinator::List(result))
            }
            Self::Table(map) => {
                let mut result = BTreeMap::new();
                for (k, v) in map {
                    result.insert(k.to_string(), v.reduce(d)?.to_combinator());
                }
                Lambda::Combinator(Combinator::Table(result))
            }
        })
    }
}

//
// Result := enum.ok.err.enum(ok)(err)
// Ok     := x.ok.err.(ok x)
// Err    := e.ok.err.(err e)
// data Result = Ok(x) | Err(e)
//
//
// Shape     := enum.circle.rectangle.enum(circle)(rectangle)
// Circle    := radius.circle.rectangle.(circle radius)
// Rectangle := width.height.circle.rectangle.(rectangle width height)
// data Shape = Circle(radius) | Rectangle(width, height)
//
// Shape     := enum.circle.rectangle.enum(circle)(rectangle)
// Circle    := radius.circle.rectangle.(circle radius)
// Rectangle := width.height.circle.rectangle.(rectangle width height)
// data Shape = Circle(radius) | Rectangle(width, height)
//
// Square := color.piece.f.(f color piece)
// type Square(color, piece)
//

// // data Result = Ok(x) | Err(e)
// #[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
// pub struct Data {
//     name: String,
//     pub cons: Vec<Constructor>,
// }
