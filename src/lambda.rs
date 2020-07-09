use crate::{compiler_error, Combinator, I, K, S};
use alloc::{
    rc::Rc,
    string::{String, ToString},
};
use core::fmt::{Display, Error, Formatter};

/// Represents a Lambda term
///
/// The Abstract Syntax Tree of the parser is directly converted into
/// these Lambda expressions, which in turn are converted to Combinator calculus.
#[derive(Clone, Debug, PartialEq)]
pub enum Lambda {
    Application(Rc<Self>, Rc<Self>),
    Abstraction(String, Rc<Self>),
    Binding(String),
    Combinator(Combinator),
}

impl Lambda {
    // Constants

    /// Substitution Combinator: λx.λy.λz.(x z)(y z)
    const S: Self = Self::Combinator(S);
    /// Constant Combinator: λx.λy.x
    const K: Self = Self::Combinator(K);
    /// Identity Combinator: λx.x
    const I: Self = Self::Combinator(I);

    // Associated Functions

    /// Construct a Lambda term that is a bound variable
    #[inline]
    pub fn var(name: impl ToString) -> Self {
        Self::Binding(name.to_string())
    }

    /// Construct a Lambda abstraction
    #[inline]
    pub fn lambda(var: impl ToString, result: Self) -> Self {
        Self::Abstraction(var.to_string(), Rc::new(result))
    }

    // Methods

    /// Apply a Lambda term to another Lambda term
    #[inline]
    pub fn applied_to(self, arg: Self) -> Self {
        Self::Application(Rc::new(self), Rc::new(arg))
    }

    /// Convert a Lambda expression to a point-free expression of ONLY combinators.
    /// This combinator expression will be returned for compilation.
    pub fn to_combinator(&self) -> Combinator {
        match self.optimize() {
            Self::Application(a, b) => a.to_combinator().applied_to(b.to_combinator()),
            Self::Binding(x) => compiler_error(format!("Free variable '{}' never defined", x)),
            Self::Combinator(c) => c,
            otherwise => compiler_error(format!("Uncompileable abstraction {}", otherwise)),
        }
    }

    /// Continuously reduce a Lambda expression until it cannot be reduced
    fn optimize(&self) -> Self {
        let mut last = self.clone();
        let mut next = last.clone().reduce();

        // If the lambda term didnt change, stop
        while next != last {
            last = next;
            next = last.reduce();
        }

        next
    }

    /// Check if a lambda expression contains a bound variable
    fn has_binding(&self, var: impl ToString) -> bool {
        let var = var.to_string();
        match self {
            Self::Application(a, b) => a.has_binding(var.clone()) || b.has_binding(var),
            Self::Abstraction(a, b) if a != &var => b.has_binding(var),
            Self::Binding(x) => &var == x,
            _ => false,
        }
    }

    /// Perform a reduction step on a Lambda expression to reduce it into a combinator expression
    fn reduce(&self) -> Self {
        match self {
            Self::Abstraction(x, y) => match (**y).clone() {
                // CASE: λx.x => I;
                Self::Binding(z) if x == &z => Self::I,
                // CASE: λx.y => K y;
                Self::Binding(z) => Self::K.applied_to(Self::var(z)).reduce(),

                // CASE: λx.(A B) => K (A B);
                // This case is ONLY applicable if 'x' is not bound in BOTH 'A' or 'B'
                // This case is also optional, reduction can be achieved without it.
                // It is not known if this case improves reduction, but I believe it does.
                Self::Application(a, b) if !a.has_binding(x) && !b.has_binding(x) => Self::K
                    .applied_to(a.reduce().applied_to(b.reduce()))
                    .reduce(),
                // CASE: λx.(A B) => S (K A) (λx.B);
                // This case is ONLY applicable if 'x' is not bound in 'A'
                // This case is absolutely necessary for reduction.
                Self::Application(a, b) if !a.has_binding(x) => Self::S
                    .applied_to(Self::K.applied_to(a.reduce()).reduce())
                    .applied_to(Self::lambda(x, b.reduce()))
                    .reduce(),
                // CASE: λx.(A B) => S (λx.A) (K B);
                // This case is ONLY applicable if 'x' is not bound in 'B'
                // This case is absolutely necessary for reduction.
                Self::Application(a, b) if !b.has_binding(x) => Self::S
                    .applied_to(Self::lambda(x, a.reduce()))
                    .applied_to(Self::K.applied_to(b.reduce()).reduce())
                    .reduce(),
                // CASE: λx.(A B) => S (λx.A) (λx.B);
                // This case is absolutely necessary for reduction.
                Self::Application(a, b) => Self::S
                    .applied_to(Self::lambda(x, a.reduce()))
                    .applied_to(Self::lambda(x, b.reduce()))
                    .reduce(),

                // CASE: λx.λy.B => λx.λy.B;
                // If 'x' is bound in the expression B, then this case is used.
                // This simply reduces the inner expressions and attempts reduction again.
                // This case MIGHT also optional, reduction probably will be achieved without it,
                // but I'm not sure.It is not known if this case improves reduction, but I believe it does.
                Self::Abstraction(a, b) if b.has_binding(x.clone()) => {
                    Self::lambda(x, Self::lambda(a, b.reduce()).reduce()).reduce()
                }
                // CASE: λx.λy.B => K (λy.B)
                // This case is not completely necessary for reduction, it is a variation of the
                // pattern below. I believe it improves reduction.
                Self::Abstraction(a, b) => Self::K.applied_to(Self::lambda(a, b.reduce()).reduce()),

                // CASE: λx.B => K B
                // This case is absolutely necessary for reduction.
                otherwise => Self::K.applied_to(otherwise.reduce()),
            },
            Self::Application(a, b) => a.reduce().applied_to(b.reduce()),
            Self::Binding(x) => Self::var(x),
            Self::Combinator(c) => Self::Combinator(c.clone()),
        }
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Abstraction(a, b) => write!(f, "λ{}.{}", a, b),
            Self::Application(a, b) => write!(f, "({})({})", a, b),
            Self::Binding(a) => write!(f, "{}", a),
            Self::Combinator(c) => write!(f, "{:?}", c),
        }
    }
}
