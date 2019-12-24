use std::convert::{ From };

pub enum Parity {
    Even = 0,
    Odd = 1
}

impl Parity {
    pub fn opposite (self) -> Parity {
        match self {
            Parity::Even => { Parity::Odd }
            Parity::Odd  => { Parity::Even }
        }
    }
}

pub type Mask = u64;
pub const MAX_STAGE : usize = 64;

pub type Row = [Bell];

macro_rules! define_int_synonymn {
    ($type:ident) => {
        #[derive(PartialEq, Copy, Clone)]
        pub struct $type (u32);

        impl From<u32> for $type {
            fn from (x : u32) -> $type {
                $type (x)
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
