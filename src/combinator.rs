use crate::compiler_error;
use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::{Debug, Display, Error, Formatter};

#[derive(Clone)]
pub enum Combinator {
    S {
        x: Option<Rc<Self>>,
        y: Option<Rc<Self>>,
    },
    B {
        x: Option<Rc<Self>>,
        y: Option<Rc<Self>>,
    },
    C {
        x: Option<Rc<Self>>,
        y: Option<Rc<Self>>,
    },
    K {
        x: Option<Rc<Self>>,
    },
    I,
    Table(BTreeMap<String, Self>),
    List(Vec<Self>),
    Number(f64),
    String(String),
    Builtin {
        name: String,
        function: Rc<dyn Fn(Self) -> Self>,
    },
    Foreign {
        name: String,
        arguments: Vec<Self>,
    },
    Nil,
}

pub const S: Combinator = Combinator::S { x: None, y: None };
pub const B: Combinator = Combinator::B { x: None, y: None };
pub const C: Combinator = Combinator::C { x: None, y: None };
pub const K: Combinator = Combinator::K { x: None };
pub const I: Combinator = Combinator::I;
#[allow(non_upper_case_globals)]
pub const Nil: Combinator = Combinator::Nil;

#[inline]
pub fn builtin(name: impl ToString, f: impl Fn(Combinator) -> Combinator + 'static) -> Combinator {
    Combinator::Builtin {
        name: name.to_string(),
        function: Rc::new(f),
    }
}

#[inline]
pub fn foreign(name: impl ToString) -> Combinator {
    Combinator::Foreign {
        name: name.to_string(),
        arguments: Vec::new(),
    }
}

impl Combinator {
    pub fn applied_to(&self, arg: Self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::S {
                x: Some(a),
                y: Some(b),
            } => {
                let first = a.applied_to(arg.clone());
                let second = b.applied_to(arg);
                first.applied_to(second)
            }
            Self::S {
                x: Some(a),
                y: None,
            } => Self::S {
                x: Some(a.clone()),
                y: Some(Rc::new(arg)),
            },
            Self::S { x: None, y: None } => Self::S {
                x: Some(Rc::new(arg)),
                y: None,
            },
            Self::S {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Substitution combinator"),

            Self::B { x: None, y: None } => Self::S {
                x: Some(Rc::new(arg)),
                y: None,
            },
            Self::B {
                x: Some(a),
                y: None,
            } => Self::B {
                x: Some(a.clone()),
                y: Some(Rc::new(arg)),
            },
            Self::B {
                x: Some(a),
                y: Some(b),
            } => a.applied_to(b.applied_to(arg)),
            Self::B {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Bluebird combinator"),

            Self::C { x: None, y: None } => Self::S {
                x: Some(Rc::new(arg)),
                y: None,
            },
            Self::C {
                x: Some(a),
                y: None,
            } => Self::C {
                x: Some(a.clone()),
                y: Some(Rc::new(arg)),
            },
            Self::C {
                x: Some(a),
                y: Some(b),
            } => a.applied_to(arg.applied_to((**b).clone())),
            Self::C {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Flip Combinator"),

            Self::K { x: Some(a) } => (**a).clone(),
            Self::K { x: None } => match arg {
                // Self::String(_) | Self::Table(_) | Self::List(_) | Self::Number(_) => arg,
                _ => Self::K {
                    x: Some(Rc::new(arg)),
                },
            },

            Self::I => arg,

            Self::Table(t) => Self::Table(t.clone()),
            Self::String(s) => Self::String(s.clone()),
            Self::List(l) => Self::List(l.clone()),
            Self::Number(n) => Self::Number(*n),
            Self::Builtin { function, .. } => function(arg),
            Self::Foreign { name, arguments } => {
                let mut result = arguments.clone();
                result.push(arg);
                Self::Foreign {
                    name: name.clone(),
                    arguments: result,
                }
            }
        }
    }
}

impl Display for Combinator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::S {
                x: Some(a),
                y: Some(b),
            } => write!(f, "S({})({})", a, b),
            Self::S {
                x: Some(a),
                y: None,
            } => write!(f, "S({})", a),
            Self::S { x: None, y: None } => write!(f, "S"),
            Self::S {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Substitution Combinator"),

            Self::K { x: Some(a) } => write!(f, "K({})", a),
            Self::K { x: None } => write!(f, "K"),

            Self::I => write!(f, "I"),

            Self::B {
                x: Some(a),
                y: Some(b),
            } => write!(f, "B({})({})", a, b),
            Self::B {
                x: Some(a),
                y: None,
            } => write!(f, "B({})", a),
            Self::B { x: None, y: None } => write!(f, "B"),
            Self::B {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Bluebird Combinator"),

            Self::C {
                x: Some(a),
                y: Some(b),
            } => write!(f, "C({})({})", a, b),
            Self::C {
                x: Some(a),
                y: None,
            } => write!(f, "C({})", a),
            Self::C { x: None, y: None } => write!(f, "C"),
            Self::C {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Flip Combinator"),

            Self::Table(t) => {
                write!(f, "{{ ")?;
                for (k, v) in t {
                    write!(f, "{:?}:{:?} ", k, v)?;
                }
                write!(f, "}}")
            }

            Self::List(l) => {
                write!(f, "[ ")?;
                for item in l {
                    write!(f, "{:?} ", item)?;
                }
                write!(f, "]")
            }

            Self::Builtin { name, .. } => write!(f, "{}", name),

            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),

            Self::Foreign { name, arguments } => {
                write!(f, "{}", name)?;
                for item in arguments {
                    write!(f, "({:?})", item)?;
                }
                Ok(())
            }
        }
    }
}

impl Debug for Combinator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::S {
                x: Some(a),
                y: Some(b),
            } => write!(f, "S({:?})({:?})", a, b),
            Self::S {
                x: Some(a),
                y: None,
            } => write!(f, "S({:?})", a),
            Self::S { x: None, y: None } => write!(f, "S"),
            Self::S {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Substitution Combinator"),

            Self::K { x: Some(a) } => write!(f, "K({:?})", a),
            Self::K { x: None } => write!(f, "K"),

            Self::I => write!(f, "I"),

            Self::B {
                x: Some(a),
                y: Some(b),
            } => write!(f, "B({:?})({:?})", a, b),
            Self::B {
                x: Some(a),
                y: None,
            } => write!(f, "B({:?})", a),
            Self::B { x: None, y: None } => write!(f, "B"),
            Self::B {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Bluebird Combinator"),

            Self::C {
                x: Some(a),
                y: Some(b),
            } => write!(f, "C({:?})({:?})", a, b),
            Self::C {
                x: Some(a),
                y: None,
            } => write!(f, "C({:?})", a),
            Self::C { x: None, y: None } => write!(f, "C"),
            Self::C {
                x: None,
                y: Some(_),
            } => compiler_error("Malformed Flip Combinator"),

            Self::Table(t) => {
                write!(f, "{{ ")?;
                for (k, v) in t {
                    write!(f, "{:?}:{:?} ", k, v)?;
                }
                write!(f, "}}")
            }

            Self::List(l) => {
                write!(f, "[ ")?;
                for item in l {
                    write!(f, "{:?} ", item)?;
                }
                write!(f, "]")
            }

            Self::Builtin { name, .. } => write!(f, "{}", name),

            Self::Number(n) => write!(f, "{:?}", n),
            Self::String(s) => write!(f, "{:?}", s),

            Self::Foreign { name, arguments } => {
                write!(f, "{}", name)?;
                for item in arguments {
                    write!(f, "({:?})", item)?;
                }
                Ok(())
            }
        }
    }
}

impl PartialEq for Combinator {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::S { x, y }, Self::S { x: a, y: b }) => a == x && y == b,
            (Self::B { x, y }, Self::B { x: a, y: b }) => a == x && y == b,
            (Self::C { x, y }, Self::C { x: a, y: b }) => a == x && y == b,
            (Self::K { x }, Self::K { x: a }) => a == x,
            (Self::I, Self::I) => true,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Table(a), Self::Table(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Builtin { name: a, .. }, Self::Builtin { name: b, .. }) => a == b,
            (
                Self::Foreign {
                    name: a,
                    arguments: b,
                },
                Self::Foreign {
                    name: x,
                    arguments: y,
                },
            ) => a == x && b == y,
            _ => false,
        }
    }
}
