use std::convert::{ From };
use std::ops::{ Mul, Not };
use crate::consts::BELL_NAMES;

#[derive(PartialEq, Debug)]
pub enum Parity {
    Even = 0,
    Odd = 1
}

impl Mul for Parity {
    type Output = Self;

    fn mul (self, other : Self) -> Self {
        match (self as usize) ^ (other as usize) {
            0 => { Parity::Even }
            1 => { Parity::Odd }
            _ => { panic! ("Unknown parity found") }
        }
    }
}

impl Not for Parity {
    type Output = Self;

    fn not (self) -> Self {
        match self {
            Parity::Even => { Parity::Odd }
            Parity::Odd  => { Parity::Even }
        }
    }
}

pub type Mask = u64;
pub type Row = [Bell];

macro_rules! define_int_synonymn {
    ($type:ident) => {
        #[derive(PartialEq, Debug, Copy, Clone)]
        pub struct $type (u32);

        impl From<u32> for $type {
            fn from (x : u32) -> $type {
                $type (x)
            }
        }
        
        impl From<i32> for $type {
            fn from (x : i32) -> $type {
                if x < 0 {
                    panic! ("Can't convert a negative number");
                }

                $type (x as u32)
            }
        }

        impl From<usize> for $type {
            fn from (x : usize) -> $type {
                $type (x as u32)
            }
        }

        impl $type {
            pub fn as_u32 (&self) -> u32 {
                self.0
            }

            pub fn as_usize (&self) -> usize {
                self.as_u32 () as usize
            }
        }
    };
}

define_int_synonymn! (Place);
define_int_synonymn! (Bell);
define_int_synonymn! (Stage);

impl From<char> for Bell {
    fn from (c : char) -> Bell {
        match BELL_NAMES.find (c) {
            Some (i) => { Bell::from (i) }
            None => { panic! ("Illegal bell name '{}'", c) }
        }
    }
}
