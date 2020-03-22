use crate::{
    ast::{Data, Function},
    lambda::Lambda,
    reduce::{Reduce, ReductionError},
};

impl Reduce<Vec<Data>> for Function {
    fn reduce(&self, d: &Vec<Data>) -> Result<Lambda, ReductionError> {
        self.get_body().reduce(d)
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
