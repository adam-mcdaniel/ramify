use crate::Combinator;
use alloc::string::String;

pub trait Target<E> {
    fn compile(&self, input: Combinator) -> Result<String, E>;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum CompilerError {}

pub struct Golang;
impl Target<CompilerError> for Golang {
    fn compile(&self, input: Combinator) -> Result<String, CompilerError> {
        match input {
            Combinator::S {
                x: Some(a),
                y: Some(b),
            } => Ok(format!(
                "S.apply({}).apply({})",
                self.compile((*a).clone())?,
                self.compile((*b).clone())?,
            )),
            Combinator::S {
                x: Some(a),
                y: None,
            } => Ok(format!("S.apply({})", self.compile((*a).clone())?,)),
            Combinator::S { x: None, y: None } => Ok(String::from("S")),

            Combinator::B {
                x: Some(a),
                y: Some(b),
            } => Ok(format!(
                "B.apply({}).apply({})",
                self.compile((*a).clone())?,
                self.compile((*b).clone())?,
            )),
            Combinator::B {
                x: Some(a),
                y: None,
            } => Ok(format!("B.apply({})", self.compile((*a).clone())?,)),
            Combinator::B { x: None, y: None } => Ok(String::from("B")),

            Combinator::C {
                x: Some(a),
                y: Some(b),
            } => Ok(format!(
                "C.apply({}).apply({})",
                self.compile((*a).clone())?,
                self.compile((*b).clone())?,
            )),
            Combinator::C {
                x: Some(a),
                y: None,
            } => Ok(format!("C.apply({})", self.compile((*a).clone())?,)),
            Combinator::C { x: None, y: None } => Ok(String::from("C")),

            Combinator::K { x: Some(a) } => {
                Ok(format!("K.apply({})", self.compile((*a).clone())?,))
            }
            Combinator::K { x: None } => Ok(String::from("K")),
            Combinator::I => Ok(String::from("I")),

            Combinator::Table(table) => {
                let mut result = String::from("make_table(map[string]Combinator {");
                for (k, v) in table {
                    result += &format!("\"{}\":{}, ", k, self.compile(v)?);
                }
                Ok(result + "})")
            }
            Combinator::List(list) => {
                let mut result = String::from("make_list([]Combinator {");
                for item in list {
                    result += &format!("{}, ", self.compile(item)?);
                }
                Ok(result + "})")
            }
            Combinator::Number(n) => Ok(format!("make_f64({})", n)),
            Combinator::String(s) => Ok(format!("make_str(\"{}\")", s)),
            Combinator::Builtin { name, .. } => Ok(name),

            Combinator::Foreign { name, arguments } => {
                let mut result = name.clone();
                for arg in &arguments {
                    result += &format!(".apply({})", self.compile(arg.clone())?);
                }
                Ok(result)
            }

            otherwise => panic!("Malformed Combinator '{}'", otherwise),
        }
    }
}
