use std::convert::{ From };
use std::ops::{ Mul, Not };
use crate::consts::BELL_NAMES;
use std::fmt;

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

type MaskType = u64;

pub struct MaskStruct {
    pub mask : MaskType
}

pub type Mask = MaskStruct;

pub trait MaskMethods {
    fn empty () -> Self;
    fn limit () -> Number;

    fn get (&self, value : Number) -> bool;
    fn del (&mut self, value : Number) -> ();
    fn add (&mut self, value : Number) -> ();
}

impl fmt::Debug for Mask {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity (Mask::limit () as usize);

        for i in 0..Mask::limit () {
            s.push (if self.get (i) { '1' } else { '1' });
        }

        write! (f, "{}", s)
    }
}

impl MaskMethods for MaskStruct {
    fn empty () -> MaskStruct {
        MaskStruct { mask : 0 as MaskType }
    }

    fn limit () -> Number {
        64
    }

    fn get (&self, value : Number) -> bool {
        self.mask & ((1 as MaskType) << value) != 0
    }

    fn del (&mut self, value : Number) -> () {
        self.mask &= !(1 as MaskType) << value
    }

    fn add (&mut self, value : Number) -> () {
        self.mask |= (1 as MaskType) << value
    }
}

#[cfg(test)]
mod mask_tests {
    use crate::types::*;

    #[test]
    fn empty_limit () {
        let mask = Mask::empty ();

        for i in 0..Mask::limit () {
            assert! (!mask.get (i));
        }
    }

    #[test]
    fn get () {
        let mask = Mask { mask : 0b10001_1000u64 };

        assert! (!mask.get (0));
        assert! (mask.get (3));
        assert! (mask.get (4));
        assert! (!mask.get (25));
    }

    #[test]
    fn add () {
        let mut mask = Mask { mask : 0b10001_1000u64 };

        assert! (!mask.get (0));
        assert! (mask.get (3));
        assert! (mask.get (4));
        assert! (!mask.get (25));

        mask.add (25);

        assert! (mask.get (25));
    }
}

pub type Number = u32;

macro_rules! define_int_synonymn {
    ($type:ident) => {
        #[derive(PartialEq, Debug, Copy, Clone)]
        pub struct $type (Number);

        impl From<Number> for $type {
            fn from (x : Number) -> $type {
                $type (x)
            }
        }
        
        impl From<i32> for $type {
            fn from (x : i32) -> $type {
                if x < 0 {
                    panic! ("Can't convert a negative number");
                }

                $type (x as Number)
            }
        }

        impl From<usize> for $type {
            fn from (x : usize) -> $type {
                $type (x as Number)
            }
        }

        impl $type {
            pub fn as_number (&self) -> Number {
                self.0 as Number
            }

            pub fn as_u32 (&self) -> u32 {
                self.0 as u32
            }

            pub fn as_usize (&self) -> usize {
                self.as_u32 () as usize
            }

            pub fn as_char (&self) -> char {
                if self.0 >= BELL_NAMES.len () as Number {
                    panic! ("Bell name '{}' too big to convert to char", self.0);
                }

                BELL_NAMES.as_bytes () [self.as_usize ()] as char
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
