use alloc::{collections::BTreeMap, rc::Rc, string::String, vec::Vec};
use core::fmt::{Debug, Error, Formatter};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AST {
    imports: Vec<Import>,
    constants: Vec<Constant>,
    cons: Vec<Constructor>,
    data: Vec<Data>,
    functions: Vec<Function>,
}

impl AST {
    pub fn new(
        imports: Vec<Import>,
        constants: Vec<Constant>,
        cons: Vec<Constructor>,
        data: Vec<Data>,
        functions: Vec<Function>,
    ) -> Self {
        Self {
            imports,
            constants,
            cons,
            data,
            functions,
        }
    }

    pub fn get_data(&self) -> &Vec<Data> {
        &self.data
    }

    pub fn get_constants(&self) -> &Vec<Constant> {
        &self.constants
    }

    pub fn get_functions(&self) -> &Vec<Function> {
        &self.functions
    }

    pub fn inline_functions(&mut self) {
        self.replace_constants();
        for _ in 0..self.functions.len() {
            let functions = self.functions.clone();
            for a in &mut self.functions {
                for b in &functions {
                    a.inline_function(b);
                }
            }
        }
        self.replace_constants();
    }

    pub fn resolve_types(&mut self) {
        for f in &mut self.functions {
            f.resolve_types(&self.data, &self.cons);
        }

        for c in &mut self.constants {
            c.resolve_types(&self.data, &self.cons);
        }
    }

    pub fn resolve_tailcalls(&mut self) {
        for f in &mut self.functions {
            f.resolve_tailcall();
        }
    }

    pub fn replace_constants(&mut self) {
        for f in &mut self.functions {
            for const1 in self.constants.clone() {
                f.replace_constant(&const1);
                for const2 in &mut self.constants {
                    const2.replace_constant(&const1);
                }
            }
        }
    }

    pub fn replace_constructors(&mut self) {
        for f in &mut self.functions {
            for data in &self.data {
                for cons in &data.cons {
                    f.replace_constructors(&cons);
                    for constant in &mut self.constants {
                        constant.replace_constructors(&cons);
                    }
                }
            }

            for cons in &self.cons {
                f.replace_constructors(&cons);
                for constant in &mut self.constants {
                    constant.replace_constructors(&cons);
                }
            }
        }
    }

    pub fn has_conflicting_datatypes(&self) -> bool {
        for a in &self.data {
            for b in &self.data {
                if a == b {
                    return true;
                }
            }
        }

        false
    }
}

// from std import True, False
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Import {
    file: String,
    items: Vec<String>,
}

impl Import {
    pub fn new(file: String, items: Vec<String>) -> Self {
        Self { file, items }
    }
}

// const True = (x y -> x)
// const False = (x y -> y)
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Constant {
    name: String,
    value: Expression,
}

impl Constant {
    pub fn new(name: String, value: Expression) -> Self {
        Self { name, value }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_body(&self) -> &Expression {
        &self.value
    }

    pub fn is_recursive(&self) -> bool {
        self.value.has_binding(&self.name)
    }

    pub fn resolve_types(&mut self, data: &Vec<Data>, cons: &Vec<Constructor>) {
        self.value = (*self.value.resolve_types(data, cons)).clone();
    }

    pub fn replace_constant(&mut self, constant: &Constant) {
        if !self.is_recursive() {
            self.value = (*self.value.replace_constant(constant)).clone();
        }
    }

    pub fn replace_constructors(&mut self, cons: &Constructor) {
        self.value = (*self.value.replace_constructors(cons)).clone();
    }
}

// data Result = Ok(x) | Err(e)
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Data {
    name: String,
    pub cons: Vec<Constructor>,
}

impl Data {
    pub fn new(name: String, cons: Vec<Constructor>) -> Self {
        Self { name, cons }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_constructors(&self) -> Vec<Constructor> {
        let mut copy = self.cons.clone();
        copy.sort();
        copy
    }
}

// Err(e)
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Constructor {
    name: String,
    members: Vec<String>,
}

impl Constructor {
    pub fn new(name: String, members: Vec<String>) -> Self {
        Self { name, members }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_members(&self) -> &Vec<String> {
        &self.members
    }
}

// let mul a b = a * b
// let factorial n = if n > 0 then (n * rec n-1) else 1
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Function {
    name: String,
    body: Expression,
}

impl Function {
    pub fn new(name: String, mut args: Vec<String>, mut body: Expression) -> Self {
        args.reverse();
        for arg in &args {
            body = Expression::Lambda(arg.clone(), Rc::new(body));
        }

        Self { name, body }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_body(&self) -> &Expression {
        &self.body
    }

    pub fn inline_function(&mut self, f: &Self) {
        self.replace_constant(&Constant::new(f.get_name().clone(), f.get_body().clone()))
    }

    pub fn resolve_tailcall(&mut self) {
        self.body = (*self.body.resolve_tailcall(true)).clone();
    }

    pub fn resolve_types(&mut self, data: &Vec<Data>, cons: &Vec<Constructor>) {
        self.body = (*self.body.resolve_types(data, cons)).clone();
    }

    pub fn replace_constant(&mut self, constant: &Constant) {
        self.body = (*self.body.replace_constant(constant)).clone();
    }

    pub fn replace_constructors(&mut self, cons: &Constructor) {
        self.body = (*self.body.replace_constructors(cons)).clone();
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, PartialOrd)]
pub enum Expression {
    // a && b
    And(Rc<Self>, Rc<Self>),
    // a || b
    Or(Rc<Self>, Rc<Self>),
    // !a
    Not(Rc<Self>),

    // a == b
    Equal(Rc<Self>, Rc<Self>),
    // a != b
    NotEqual(Rc<Self>, Rc<Self>),
    // a > b
    Greater(Rc<Self>, Rc<Self>),
    // a >= b
    GreaterEqual(Rc<Self>, Rc<Self>),
    // a < b
    Less(Rc<Self>, Rc<Self>),
    // a <= b
    LessEqual(Rc<Self>, Rc<Self>),

    // -a
    Negate(Rc<Self>),
    // a + b
    Add(Rc<Self>, Rc<Self>),
    // a * b
    Multiply(Rc<Self>, Rc<Self>),
    // a / b
    Divide(Rc<Self>, Rc<Self>),
    // a - b
    Subtract(Rc<Self>, Rc<Self>),

    // if c then a else b
    IfThenElse {
        condition: Rc<Self>,
        then_case: Rc<Self>,
        else_case: Rc<Self>,
    },

    // case input "> " of
    //  | Ok(val) => print "You said: " val
    //  | Err(_)  => print "There was a problem retrieving input"
    CaseOf {
        /// This field is optional because the data type is derived from
        /// the constructors. Likely, the first compiler pass will not
        /// be able to determine the data type of the constructor cases.
        data_type: Option<Data>,
        /// The value to scrutinize
        value: Rc<Self>,
        /// Constructor name, constructor arguments, body of case
        ///
        /// Name Arguments Body of case
        ///  |     |         |
        ///  v v---/         v
        /// Ok(x) => print "You said: " x
        cases: Vec<(String, Vec<String>, Rc<Self>)>,
    },

    // Point(1, 2)
    Construct {
        /// This field is optional because this will be filled in by the AST
        /// transformations. The first compiler pass may not fill this field
        /// the first pass.
        data_type: Option<Constructor>,
        /// The name of the constructor being filled
        cons_name: String,
        /// The arguments to the data constructor
        members: Vec<Rc<Self>>,
    },

    // let Point(x, y) = p in print "(" x "," y ")"
    Deconstruct {
        /// This field is optional because this will be filled in by the AST
        /// transformations. The first compiler pass may not fill this field
        /// the first pass.
        data_type: Option<Constructor>,
        /// The name of the type being filled
        cons_name: String,
        /// The arguments to the data constructor
        members: Vec<String>,
        /// The value to deconstruct
        value: Rc<Self>,
        /// The body of the let expression
        body: Rc<Self>,
    },

    Identifier(String),
    // x z (y z)
    Application(Rc<Self>, Rc<Self>),
    // (x y -> x)
    Lambda(String, Rc<Self>),
    // rec n-1
    TailCall(Vec<Rc<Self>>),

    // { "a": 1, "b": [2, 3, 5, 7, 11] }
    Table(BTreeMap<String, Rc<Self>>),
    // [1, 2, 3]
    List(Vec<Rc<Self>>),
    // 2.5
    Number(f64),
    // "Hello world!"
    String(String),
}

impl Expression {
    pub(crate) const RECURSION_ARGUMENT: &'static str = "rec";

    pub fn y_combinator() -> Self {
        // Self::Lambda(
        //     String::from("f"),
        //     Rc::new(Expression::Application(
        //         Rc::new(Self::Lambda(
        //             String::from("x"),
        //             Rc::new(Expression::Application(
        //                 Rc::new(Expression::Identifier(String::from("f"))),
        //                 Rc::new(Expression::Application(
        //                     Rc::new(Expression::Identifier(String::from("x"))),
        //                     Rc::new(Expression::Identifier(String::from("x"))),
        //                 )),
        //             )),
        //         )),
        //         Rc::new(Self::Lambda(
        //             String::from("x"),
        //             Rc::new(Expression::Application(
        //                 Rc::new(Expression::Identifier(String::from("f"))),
        //                 Rc::new(Expression::Application(
        //                     Rc::new(Expression::Identifier(String::from("x"))),
        //                     Rc::new(Expression::Identifier(String::from("x"))),
        //                 )),
        //             )),
        //         )),
        //     )),
        // )
        Self::Identifier(String::from("Y"))
    }

    pub fn number_of_arguments(&self) -> i32 {
        match self {
            Self::Lambda(_, b) => b.number_of_arguments() + 1,
            Self::Application(a, _) => {
                if let Self::Lambda(_, _) = (**a).clone() {
                    a.number_of_arguments() - 1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    pub fn has_binding(&self, name: &String) -> bool {
        match self {
            Self::And(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::Or(a, b) => a.has_binding(name) || b.has_binding(name),

            Self::Not(a) => a.has_binding(name),

            Self::Equal(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::NotEqual(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::Greater(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::GreaterEqual(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::Less(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::LessEqual(a, b) => a.has_binding(name) || b.has_binding(name),

            Self::Negate(a) => a.has_binding(name),
            Self::Add(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::Multiply(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::Divide(a, b) => a.has_binding(name) || b.has_binding(name),
            Self::Subtract(a, b) => a.has_binding(name) || b.has_binding(name),

            Self::IfThenElse {
                condition,
                then_case,
                else_case,
            } => {
                condition.has_binding(name)
                    || then_case.has_binding(name)
                    || else_case.has_binding(name)
            }
            Self::CaseOf { value, cases, .. } => {
                for (_, _, case_body) in (*cases).clone() {
                    if case_body.has_binding(name) {
                        return true;
                    }
                }
                value.has_binding(name)
            }
            Self::Construct { members, .. } => {
                for member in (*members).clone() {
                    if member.has_binding(name) {
                        return true;
                    }
                }
                false
            }
            Self::Deconstruct { value, body, .. } => {
                body.has_binding(name) || value.has_binding(name)
            }

            Self::Lambda(_, b) => b.has_binding(name),
            Self::Application(a, b) => a.has_binding(name) || b.has_binding(name),

            Self::Table(map) => {
                for v in map.values() {
                    if v.has_binding(name) {
                        return true;
                    }
                }
                false
            }

            Self::List(list) => {
                for v in list {
                    if v.has_binding(name) {
                        return true;
                    }
                }
                false
            }

            Self::Identifier(n) => n == name,
            Self::TailCall(items) => {
                for item in items {
                    if item.has_binding(name) {
                        return true;
                    }
                }
                false
            }

            _ => false,
        }
    }

    pub fn resolve_tailcall(&self, is_head: bool) -> Rc<Self> {
        if !self.is_recursive() {
            return Rc::new(self.clone());
        }

        if is_head {
            let result = Rc::new(Self::Application(
                Rc::new(Self::y_combinator()),
                Rc::new(Self::Lambda(
                    String::from(Self::RECURSION_ARGUMENT),
                    self.resolve_tailcall(false),
                )),
            ));
            result
        } else {
            println!("  => {:?}", self);
            Rc::new(match self {
                Self::And(a, b) => Self::And(a.resolve_tailcall(false), b.resolve_tailcall(false)),
                Self::Or(a, b) => Self::Or(a.resolve_tailcall(false), b.resolve_tailcall(false)),
                Self::Not(a) => Self::Not(a.resolve_tailcall(false)),
                Self::Equal(a, b) => {
                    Self::Equal(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::NotEqual(a, b) => {
                    Self::NotEqual(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::Greater(a, b) => {
                    Self::Greater(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::GreaterEqual(a, b) => {
                    Self::GreaterEqual(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::Less(a, b) => {
                    Self::Less(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::LessEqual(a, b) => {
                    Self::LessEqual(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::Negate(a) => Self::Negate(a.resolve_tailcall(false)),
                Self::Add(a, b) => Self::Add(a.resolve_tailcall(false), b.resolve_tailcall(false)),
                Self::Multiply(a, b) => {
                    Self::Multiply(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::Divide(a, b) => {
                    Self::Divide(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::Subtract(a, b) => {
                    Self::Subtract(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::IfThenElse {
                    condition,
                    then_case,
                    else_case,
                } => Self::IfThenElse {
                    condition: condition.resolve_tailcall(false),
                    then_case: then_case.resolve_tailcall(false),
                    else_case: else_case.resolve_tailcall(false),
                },
                Self::CaseOf {
                    value: v,
                    cases,
                    data_type,
                } => {
                    let mut result = Vec::new();
                    for (a, b, case_body) in (*cases).clone() {
                        result.push((a, b, case_body.resolve_tailcall(false)));
                    }
                    Self::CaseOf {
                        data_type: (*data_type).clone(),
                        value: v.resolve_tailcall(false),
                        cases: result,
                    }
                }
                Self::Construct {
                    cons_name,
                    members,
                    data_type,
                } => {
                    let mut result = Vec::new();
                    for member in (*members).clone() {
                        result.push(member.resolve_tailcall(false));
                    }
                    Self::Construct {
                        data_type: (*data_type).clone(),
                        cons_name: cons_name.clone(),
                        members: result,
                    }
                }
                Self::Deconstruct {
                    cons_name,
                    members,
                    data_type,
                    body,
                    value,
                } => Self::Deconstruct {
                    data_type: (*data_type).clone(),
                    cons_name: cons_name.clone(),
                    members: members.clone(),
                    value: value.resolve_tailcall(false),
                    body: body.resolve_tailcall(false),
                },
                // Self::Lambda(_, _) => Self::Application(
                //     Rc::new(Self::Identifier(String::from("Y"))),
                //     Rc::new(Self::Lambda(
                //         String::from(Self::RECURSION_ARGUMENT),
                //         self.resolve_tailcall(false),
                //     )),
                // ),
                Self::Lambda(a, b) => Self::Lambda(a.clone(), b.resolve_tailcall(false)),
                Self::Application(a, b) => {
                    Self::Application(a.resolve_tailcall(false), b.resolve_tailcall(false))
                }
                Self::Table(map) => {
                    let mut t = BTreeMap::new();
                    for (k, v) in (*map).clone() {
                        t.insert(k, v.resolve_tailcall(false));
                    }
                    Self::Table(t)
                }
                Self::List(list) => {
                    let mut result = Vec::new();
                    for item in list {
                        result.push(item.resolve_tailcall(false));
                    }
                    Self::List(result)
                }
                Self::TailCall(items) => {
                    let mut args = Vec::new();
                    for item in items {
                        args.push(item.resolve_tailcall(false));
                    }

                    let mut result = Self::Identifier(String::from(Self::RECURSION_ARGUMENT));
                    for arg in args {
                        result = Self::Application(Rc::new(result), arg);
                    }

                    result
                }
                _ => self.clone(),
            })
        }
    }

    pub fn replace_constant(&self, constant: &Constant) -> Rc<Self> {
        Rc::new(match self {
            Self::And(a, b) => {
                Self::And(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::Or(a, b) => Self::Or(a.replace_constant(constant), b.replace_constant(constant)),

            Self::Not(a) => Self::Not(a.replace_constant(constant)),

            Self::Equal(a, b) => {
                Self::Equal(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::NotEqual(a, b) => {
                Self::NotEqual(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::Greater(a, b) => {
                Self::Greater(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::GreaterEqual(a, b) => {
                Self::GreaterEqual(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::Less(a, b) => {
                Self::Less(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::LessEqual(a, b) => {
                Self::LessEqual(a.replace_constant(constant), b.replace_constant(constant))
            }

            Self::Negate(a) => Self::Negate(a.replace_constant(constant)),
            Self::Add(a, b) => {
                Self::Add(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::Multiply(a, b) => {
                Self::Multiply(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::Divide(a, b) => {
                Self::Divide(a.replace_constant(constant), b.replace_constant(constant))
            }
            Self::Subtract(a, b) => {
                Self::Subtract(a.replace_constant(constant), b.replace_constant(constant))
            }

            Self::IfThenElse {
                condition,
                then_case,
                else_case,
            } => Self::IfThenElse {
                condition: condition.replace_constant(constant),
                then_case: then_case.replace_constant(constant),
                else_case: else_case.replace_constant(constant),
            },
            Self::CaseOf {
                value: v,
                cases,
                data_type,
            } => {
                let mut result = Vec::new();
                for (a, b, case_body) in (*cases).clone() {
                    result.push((a, b, case_body.replace_constant(constant)));
                }
                Self::CaseOf {
                    data_type: (*data_type).clone(),
                    value: v.replace_constant(constant),
                    cases: result,
                }
            }
            Self::Construct {
                data_type,
                members,
                cons_name,
            } => {
                let mut result = Vec::new();
                for member in (*members).clone() {
                    result.push(member.replace_constant(constant));
                }
                Self::Construct {
                    data_type: (*data_type).clone(),
                    cons_name: cons_name.clone(),
                    members: result,
                }
            }
            Self::Deconstruct {
                data_type,
                members,
                cons_name,
                value,
                body,
            } => Self::Deconstruct {
                data_type: (*data_type).clone(),
                cons_name: cons_name.clone(),
                members: members.clone(),
                value: value.replace_constant(constant),
                body: body.replace_constant(constant),
            },

            Self::Lambda(a, b) => Self::Lambda(a.clone(), b.replace_constant(constant)),
            Self::Application(a, b) => {
                Self::Application(a.replace_constant(constant), b.replace_constant(constant))
            }

            Self::Table(map) => {
                let mut t = BTreeMap::new();
                for (k, v) in (*map).clone() {
                    t.insert(k, v.replace_constant(constant));
                }
                Self::Table(t)
            }

            Self::List(list) => {
                let mut result = Vec::new();
                for item in list {
                    result.push(item.replace_constant(constant));
                }

                Self::List(result)
            }

            Self::Identifier(name) if name.clone() == constant.name => constant.value.clone(),
            Self::TailCall(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(item.replace_constant(constant));
                }

                Self::TailCall(result)
            }

            _ => self.clone(),
        })
    }

    pub fn replace_constructors(&self, cons: &Constructor) -> Rc<Self> {
        Rc::new(match self {
            Self::And(a, b) => {
                Self::And(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::Or(a, b) => Self::Or(a.replace_constructors(cons), b.replace_constructors(cons)),

            Self::Not(a) => Self::Not(a.replace_constructors(cons)),

            Self::Equal(a, b) => {
                Self::Equal(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::NotEqual(a, b) => {
                Self::NotEqual(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::Greater(a, b) => {
                Self::Greater(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::GreaterEqual(a, b) => {
                Self::GreaterEqual(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::Less(a, b) => {
                Self::Less(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::LessEqual(a, b) => {
                Self::LessEqual(a.replace_constructors(cons), b.replace_constructors(cons))
            }

            Self::Negate(a) => Self::Negate(a.replace_constructors(cons)),
            Self::Add(a, b) => {
                Self::Add(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::Multiply(a, b) => {
                Self::Multiply(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::Divide(a, b) => {
                Self::Divide(a.replace_constructors(cons), b.replace_constructors(cons))
            }
            Self::Subtract(a, b) => {
                Self::Subtract(a.replace_constructors(cons), b.replace_constructors(cons))
            }

            Self::IfThenElse {
                condition,
                then_case,
                else_case,
            } => Self::IfThenElse {
                condition: condition.replace_constructors(cons),
                then_case: then_case.replace_constructors(cons),
                else_case: else_case.replace_constructors(cons),
            },
            Self::CaseOf {
                value: v,
                cases,
                data_type,
            } => {
                let mut result = Vec::new();
                for (a, b, case_body) in (*cases).clone() {
                    result.push((a, b, case_body.replace_constructors(cons)));
                }
                Self::CaseOf {
                    data_type: (*data_type).clone(),
                    value: v.replace_constructors(cons),
                    cases: result,
                }
            }
            Self::Construct {
                data_type,
                members,
                cons_name,
            } => {
                let mut result = Vec::new();
                for member in (*members).clone() {
                    result.push(member.replace_constructors(cons));
                }
                Self::Construct {
                    data_type: (*data_type).clone(),
                    cons_name: cons_name.clone(),
                    members: result,
                }
            }
            Self::Deconstruct {
                data_type,
                members,
                cons_name,
                value,
                body,
            } => Self::Deconstruct {
                data_type: (*data_type).clone(),
                cons_name: cons_name.clone(),
                members: members.clone(),
                value: value.replace_constructors(cons),
                body: body.replace_constructors(cons),
            },

            Self::Lambda(a, b) => Self::Lambda(a.clone(), b.replace_constructors(cons)),
            Self::Application(a, b) => {
                if let Self::Identifier(name) = (**a).clone() {
                    if cons.name == name && cons.members.len() == 1 {
                        return Rc::new(Self::Construct {
                            data_type: Some(cons.clone()),
                            cons_name: cons.name.clone(),
                            members: vec![b.clone()],
                        });
                    }
                }

                Self::Application(a.replace_constructors(cons), b.replace_constructors(cons))
            }

            Self::Table(map) => {
                let mut t = BTreeMap::new();
                for (k, v) in (*map).clone() {
                    t.insert(k, v.replace_constructors(cons));
                }
                Self::Table(t)
            }

            Self::List(list) => {
                let mut result = Vec::new();
                for item in list {
                    result.push(item.replace_constructors(cons));
                }

                Self::List(result)
            }

            Self::Identifier(name) => {
                if &cons.name == name && cons.members.is_empty() {
                    return Rc::new(Self::Construct {
                        data_type: Some(cons.clone()),
                        cons_name: cons.name.clone(),
                        members: Vec::new(),
                    });
                }
                Self::Identifier(name.clone())
            }
            Self::TailCall(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(item.replace_constructors(cons));
                }

                Self::TailCall(result)
            }

            _ => self.clone(),
        })
    }

    pub fn resolve_types(&self, data: &Vec<Data>, cons: &Vec<Constructor>) -> Rc<Self> {
        Rc::new(match self {
            Self::And(a, b) => Self::And(a.resolve_types(data, cons), b.resolve_types(data, cons)),
            Self::Or(a, b) => Self::Or(a.resolve_types(data, cons), b.resolve_types(data, cons)),

            Self::Not(a) => Self::Not(a.resolve_types(data, cons)),

            Self::Equal(a, b) => {
                Self::Equal(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }
            Self::NotEqual(a, b) => {
                Self::NotEqual(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }
            Self::Greater(a, b) => {
                Self::Greater(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }
            Self::GreaterEqual(a, b) => {
                Self::GreaterEqual(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }
            Self::Less(a, b) => {
                Self::Less(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }
            Self::LessEqual(a, b) => {
                Self::LessEqual(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }

            Self::Negate(a) => Self::Negate(a.resolve_types(data, cons)),
            Self::Add(a, b) => Self::Add(a.resolve_types(data, cons), b.resolve_types(data, cons)),
            Self::Multiply(a, b) => {
                Self::Multiply(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }
            Self::Divide(a, b) => {
                Self::Divide(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }
            Self::Subtract(a, b) => {
                Self::Subtract(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }

            Self::IfThenElse {
                condition,
                then_case,
                else_case,
            } => Self::IfThenElse {
                condition: condition.resolve_types(data, cons),
                then_case: then_case.resolve_types(data, cons),
                else_case: else_case.resolve_types(data, cons),
            },
            Self::CaseOf { value, cases, .. } => {
                for d in data {
                    let mut check = false;
                    for (name, members, _) in cases {
                        let num_members = members.len();
                        check = false;
                        for con in d.get_constructors() {
                            if con.get_name() == name && con.get_members().len() == num_members {
                                check = true;
                            }
                        }
                        if !check {
                            break;
                        }
                    }

                    if check {
                        return Rc::new(Self::CaseOf {
                            value: value.clone(),
                            cases: cases.clone(),
                            data_type: Some(d.clone()),
                        });
                    }
                }
                self.clone()
                // let mut result = Vec::new();
                // for (a, b, case_body) in (*cases).clone() {
                //     result.push((a, b, case_body.resolve_types(data, cons)));
                // }
                // Self::CaseOf {
                //     data_type: (*data_type).clone(),
                //     value: v.resolve_types(data, cons),
                //     cases: result,
                // }
            }
            Self::Construct {
                members, cons_name, ..
            } => {
                for con in cons {
                    if cons_name == con.get_name() && con.get_members().len() == members.len() {
                        return Rc::new(Self::Construct {
                            data_type: Some(con.clone()),
                            members: members.clone(),
                            cons_name: cons_name.clone(),
                        });
                    }
                }
                for d in data {
                    for con in d.get_constructors() {
                        if cons_name == con.get_name() && con.get_members().len() == members.len() {
                            return Rc::new(Self::Construct {
                                data_type: Some(con.clone()),
                                members: members.clone(),
                                cons_name: cons_name.clone(),
                            });
                        }
                    }
                }
                self.clone()
            }
            Self::Deconstruct {
                data_type,
                members,
                cons_name,
                value,
                body,
            } => Self::Deconstruct {
                data_type: (*data_type).clone(),
                cons_name: cons_name.clone(),
                members: members.clone(),
                value: value.resolve_types(data, cons),
                body: body.resolve_types(data, cons),
            },

            Self::Lambda(a, b) => Self::Lambda(a.clone(), b.resolve_types(data, cons)),
            Self::Application(a, b) => {
                // if let Self::Identifier(name) = (**a).clone() {
                //     if cons.name == name && cons.members.len() == 1 {
                //         return Rc::new(Self::Construct {
                //             data_type: Some(cons.clone()),
                //             cons_name: cons.name.clone(),
                //             members: vec![b.clone()],
                //         });
                //     }
                // }

                Self::Application(a.resolve_types(data, cons), b.resolve_types(data, cons))
            }

            Self::Table(map) => {
                let mut t = BTreeMap::new();
                for (k, v) in (*map).clone() {
                    t.insert(k, v.resolve_types(data, cons));
                }
                Self::Table(t)
            }

            Self::List(list) => {
                let mut result = Vec::new();
                for item in list {
                    result.push(item.resolve_types(data, cons));
                }

                Self::List(result)
            }

            Self::TailCall(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(item.resolve_types(data, cons));
                }

                Self::TailCall(result)
            }

            _ => self.clone(),
        })
    }

    /// Visit each node of an expression and check for any tail calls
    pub fn is_recursive(&self) -> bool {
        match self {
            Self::And(a, b) => a.is_recursive() || b.is_recursive(),
            Self::Or(a, b) => a.is_recursive() || b.is_recursive(),
            Self::Not(a) => a.is_recursive(),

            Self::Equal(a, b) => a.is_recursive() || b.is_recursive(),
            Self::NotEqual(a, b) => a.is_recursive() || b.is_recursive(),
            Self::Greater(a, b) => a.is_recursive() || b.is_recursive(),
            Self::GreaterEqual(a, b) => a.is_recursive() || b.is_recursive(),
            Self::Less(a, b) => a.is_recursive() || b.is_recursive(),
            Self::LessEqual(a, b) => a.is_recursive() || b.is_recursive(),

            Self::Negate(a) => a.is_recursive(),
            Self::Add(a, b) => a.is_recursive() || b.is_recursive(),
            Self::Multiply(a, b) => a.is_recursive() || b.is_recursive(),
            Self::Divide(a, b) => a.is_recursive() || b.is_recursive(),
            Self::Subtract(a, b) => a.is_recursive() || b.is_recursive(),

            Self::IfThenElse {
                condition,
                then_case,
                else_case,
            } => condition.is_recursive() || then_case.is_recursive() || else_case.is_recursive(),
            Self::CaseOf { value, cases, .. } => {
                for (_, _, case_body) in cases {
                    if case_body.is_recursive() {
                        return true;
                    }
                }
                value.is_recursive()
            }

            Self::Construct { members, .. } => {
                for member in members {
                    if member.is_recursive() {
                        return true;
                    }
                }
                false
            }
            Self::Deconstruct { value, body, .. } => value.is_recursive() || body.is_recursive(),
            Self::Lambda(_, a) => a.is_recursive(),
            Self::Application(a, b) => a.is_recursive() || b.is_recursive(),
            Self::TailCall(_) => true,
            _ => false,
        }
    }
}

impl Debug for Expression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Identifier(i) => write!(f, "{}", i),

            Self::And(a, b) => write!(f, "({:?} && {:?})", a, b),
            Self::Or(a, b) => write!(f, "({:?} || {:?})", a, b),
            Self::Not(a) => write!(f, "!({:?})", a),

            Self::Equal(a, b) => write!(f, "({:?} == {:?})", a, b),
            Self::NotEqual(a, b) => write!(f, "({:?} != {:?})", a, b),
            Self::Greater(a, b) => write!(f, "({:?} > {:?})", a, b),
            Self::GreaterEqual(a, b) => write!(f, "({:?} >= {:?})", a, b),
            Self::Less(a, b) => write!(f, "({:?} < {:?})", a, b),
            Self::LessEqual(a, b) => write!(f, "({:?} <= {:?})", a, b),

            Self::Negate(a) => write!(f, "-({:?})", a),
            Self::Add(a, b) => write!(f, "({:?} + {:?})", a, b),
            Self::Multiply(a, b) => write!(f, "({:?} * {:?})", a, b),
            Self::Subtract(a, b) => write!(f, "({:?} - {:?})", a, b),
            Self::Divide(a, b) => write!(f, "({:?} / {:?})", a, b),

            Self::IfThenElse {
                condition,
                then_case,
                else_case,
            } => write!(
                f,
                "if ({:?}) then ({:?}) else ({:?})",
                condition, then_case, else_case
            ),

            Self::CaseOf { value, cases, .. } => {
                write!(f, "case ({:?}) of", value)?;
                for (cons, args, case) in cases {
                    write!(f, "\n\t| {}(", cons)?;
                    for arg in args {
                        write!(f, "{}, ", arg)?;
                    }
                    write!(f, ") => {:?}", case)?;
                }
                Ok(())
            }

            // Point(1, 2)
            Self::Construct {
                cons_name, members, ..
            } => {
                write!(f, "{}(", cons_name)?;

                for arg in members {
                    write!(f, "{:?}, ", arg)?;
                }
                write!(f, ")")
            }
            Self::Deconstruct {
                cons_name,
                members,
                value,
                body,
                ..
            } => {
                write!(f, "let {}(", cons_name)?;
                for arg in members {
                    write!(f, "{}, ", arg)?;
                }
                write!(f, ") = {:?} in ({:?})", value, body)
            }

            Self::Lambda(a, b) => {
                if let Self::Lambda(_, _) = (**b).clone() {
                    write!(f, "{}.{:?}", a, b)
                } else {
                    write!(f, "{}.({:?})", a, b)
                }
            }
            Self::Application(a, b) => write!(f, "{:?}({:?})", a, b),
            Self::Table(map) => write!(f, "{:?}", map),
            Self::List(list) => write!(f, "{:?}", list),
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::TailCall(items) => {
                write!(f, "rec ")?;
                for item in items {
                    write!(f, "{:?}, ", item)?;
                }
                Ok(())
            }
        }
    }
}
