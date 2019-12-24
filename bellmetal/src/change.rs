use crate::types::{ Bell, Stage, Parity };
use core::ops::{ Mul, Not };

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

impl Not for Change {
    type Output = Self;

    fn not (self) -> Self {
        let mut new_seq : Vec<Bell> = vec![Bell::from (0u32); self.stage ().as_usize ()];

        for i in 0..self.stage ().as_usize () {
            new_seq [self.seq [i as usize].as_usize ()] = Bell::from (i);
        }

        Change { seq : new_seq }
    }
}

