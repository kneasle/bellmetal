use crate::types::{ Bell, Stage, Parity };
use core::ops::Mul;

pub struct Change {
    seq : Vec<Bell>
}

impl Change {
    fn stage (&self) -> Stage {
        Stage::from (self.seq.len ())
    }

    fn parity (&self) -> Parity {
        Parity::Even
    }
}

impl Mul for Change {
    type Output = Self;

    fn mul (self, rhs : Self) -> Self {
        if self.stage () != rhs.stage () {
            panic! ("Can't multiply changes of different stages!");
        }

        let mut new_seq : Vec<Bell> = Vec::with_capacity (self.stage ().as_usize ());

        for i in 0..self.stage ().as_usize () {
            new_seq.push (self.seq [rhs.seq [i].as_usize ()]);
        }

        Change { seq : new_seq }
    }
}
