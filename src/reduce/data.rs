use crate::{
    ast::Data,
    lambda::Lambda,
    reduce::{Reduce, ReductionError},
};

impl Reduce<()> for Data {
    fn reduce(&self, _: &()) -> Result<Lambda, ReductionError> {
        let mut cons = self.get_constructors();

        let mut result = Lambda::var("enum");
        for con in &cons {
            result = result.applied_to(Lambda::var(con.get_name()));
        }
        cons.reverse();
        for con in &cons {
            result = Lambda::lambda(con.get_name(), result);
        }

        Ok(Lambda::lambda("enum", result))
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
