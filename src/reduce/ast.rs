use crate::{
    ast::{Expression, Function, AST},
    lambda::Lambda,
    reduce::{Reduce, ReductionError},
};

impl Reduce<()> for AST {
    fn reduce(&self, d: &()) -> Result<Lambda, ReductionError> {
        let bad = String::from("bad");
        let mut result = Function::new(
            bad.clone(),
            vec![bad.clone()],
            Expression::Identifier(bad.clone()),
        );

        for function in self.get_functions() {
            if function.get_name() == "main" {
                result = function.clone();
            }
        }

        Ok(result.reduce(self.get_data())?)
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
// Shape(Circle(5))
//      (x.(x*x*pi))
//      (w.h.w*h)
// case Circle(5) of
//    | Circle(x) => x * x * pi
//    | Rectangle(w, h) => w * h
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
