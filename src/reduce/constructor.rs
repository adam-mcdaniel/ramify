use crate::{
    ast::{Constructor, Data},
    lambda::Lambda,
    reduce::{Reduce, ReductionError},
};

impl Reduce<Data> for Constructor {
    // Result := enum.ok.err.enum(ok)(err)
    // Ok     := x.ok.err.(ok x)
    // Err    := e.ok.err.(err e)
    // data Result = Ok(x) | Err(e)
    fn reduce(&self, data: &Data) -> Result<Lambda, ReductionError> {
        let mut members = self.get_members().clone();
        let mut cons = data.get_constructors();
        cons.reverse();

        let mut result = Lambda::var(self.get_name());
        for member in &members {
            result = result.applied_to(Lambda::var(member));
        }

        for con in cons {
            result = Lambda::lambda(con.get_name(), result);
        }

        members.reverse();
        for member in members {
            result = Lambda::lambda(member, result);
        }

        Ok(result)
    }
}

impl Reduce<()> for Constructor {
    // Square := color.piece.f.(f color piece)
    // type Square(color, piece)
    fn reduce(&self, _: &()) -> Result<Lambda, ReductionError> {
        let mut members = self.get_members().clone();

        let mut result = Lambda::var("f");
        for member in &members {
            result = result.applied_to(Lambda::var(member));
        }

        result = Lambda::lambda("f", result);

        members.reverse();
        for member in members {
            result = Lambda::lambda(member, result);
        }

        // println!("type {} = {}", self.get_name(), result);

        Ok(result)
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
