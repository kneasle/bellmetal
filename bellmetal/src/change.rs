use crate::types::*;
use core::ops::{ Mul, Not };
use std::convert::{ From };

pub struct Change {
    seq : Vec<Bell>
}

impl Change {
    fn stage (&self) -> Stage {
        Stage::from (self.seq.len ())
    }

    fn parity (&self) -> Parity {
        let mut mask : Mask = 0 as Mask;
        let mut bells_fixed = 0;

        let mut total_cycle_length = 0;

        let stage = self.stage ().as_u32 ();

        while bells_fixed < stage {
            let mut bell = 0;

            while mask & ((1 as Mask) << bell) != 0 as Mask {
                bell += 1;
            }

            total_cycle_length += 1; // Make sure that the parity is correct

            while mask & ((1 as Mask) << bell) == 0 as Mask {
                mask |= (1 as Mask) << bell;

                bell = self.seq [bell as usize].as_u32 ();

                total_cycle_length += 1;
                bells_fixed += 1;
            }
        }
        
        match total_cycle_length & 1 {
            0 => { Parity::Even },
            1 => { Parity::Odd }
            _ => { panic! ("Unknown parity") }
        }
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

impl From<&str> for Change {
    fn from (s : &str) -> Change {
        let mut new_seq : Vec<Bell> = Vec::with_capacity (s.len ());

        for c in s.chars () {
            new_seq.push (Bell::from (c));
        }

        Change { seq : new_seq }
    }
}
