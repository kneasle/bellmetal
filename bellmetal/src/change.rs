use crate::types::*;
use core::ops::{ Mul, Not };
use std::convert::{ From };
use std::fmt;

#[derive(PartialEq)]
pub struct Change {
    pub seq : Vec<Bell>
}

impl Change {
    fn stage (&self) -> Stage {
        Stage::from (self.seq.len ())
    }

    fn parity (&self) -> Parity {
        let mut mask = Mask::empty ();
        let mut bells_fixed = 0;

        let mut total_cycle_length = 0;

        let stage = self.stage ().as_number ();

        while bells_fixed < stage {
            let mut bell = 0;
                
            while mask.get (bell) {
                bell += 1;
            }

            total_cycle_length += 1; // Make sure that the parity is correct

            while !mask.get (bell) {
                mask.add (bell);
                
                bell = self.seq [bell as usize].as_number ();

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

    pub fn rounds (stage : Stage) -> Change {
        let mut seq : Vec<Bell> = Vec::with_capacity (stage.as_usize ());

        for i in 0..stage.as_usize () {
            seq.push (Bell::from (i));
        }

        Change { seq : seq }
    }
}

impl fmt::Debug for Change {
    fn fmt (&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity (self.stage ().as_usize ());

        for b in &self.seq {
            s.push (b.as_char ());
        }

        write! (f, "<{}>", s)
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

#[cfg(test)]
mod change_tests {
    use crate::change::Change;
    use crate::types::{ Bell, Stage, Parity };
    
    use std::fmt::Write;

    #[test]
    fn equality () {
        assert! (
            Change { seq : vec![
                Bell::from (1), 
                Bell::from (0), 
                Bell::from (3), 
                Bell::from (2)
            ] } 
            == 
            Change { seq : vec![
                Bell::from (1), 
                Bell::from (0), 
                Bell::from (3), 
                Bell::from (2)
            ] }
        );
        
        // Different bells
        assert! (
            Change { seq : vec![
                Bell::from (1), 
                Bell::from (0), 
                Bell::from (2), 
                Bell::from (3)
            ] } 
            != 
            Change { seq : vec![
                Bell::from (1), 
                Bell::from (0), 
                Bell::from (3), 
                Bell::from (2)
            ] }
        );
        
        // Different stage
        assert! (
            Change { seq : vec![
                Bell::from (1), 
                Bell::from (0), 
                Bell::from (3), 
                Bell::from (2),
                Bell::from (4)
            ] } 
            != 
            Change { seq : vec![
                Bell::from (1), 
                Bell::from (0), 
                Bell::from (3), 
                Bell::from (2)
            ] }
        );
    }

    #[test]
    fn from_string () {
        // Different bells
        assert_eq! (
            Change::from ("2143"),
            Change { seq : vec![
                Bell::from (1), 
                Bell::from (0), 
                Bell::from (3), 
                Bell::from (2)
            ] }
        );
        assert_eq! (
            Change::from (""),
            Change { seq : vec![] }
        );

    }

    #[test]
    #[should_panic]
    fn from_string_illegal_bell () {
        Change::from ("2134 ");
    }

    #[test]
    fn stage () {
        assert_eq! (Stage::from (0), Change::from ("").stage ());
        assert_eq! (Stage::from (1), Change::from ("1").stage ());
        assert_eq! (Stage::from (10), Change::from ("6789052431").stage ());
    }
    
    #[test]
    fn parity () {
        assert_eq! (Parity::Even, Change::from ("1234567").parity ());
        assert_eq! (Parity::Even, Change::from ("87654321").parity ());
        assert_eq! (Parity::Even, Change::from ("13425678").parity ());

        assert_eq! (Parity::Odd, Change::from ("1234657").parity ());
        assert_eq! (Parity::Odd, Change::from ("2143657890").parity ());
        assert_eq! (Parity::Odd, Change::from ("7654321").parity ());
    }

    #[test]
    fn multiplication () {
        assert_eq! (Change::from ("1324") * Change::from ("4231"), Change::from ("4321"));
        assert_eq! (Change::from ("13425678") * Change::from ("13425678"), Change::from ("14235678"));
        assert_eq! (Change::from ("543216") * Change::from ("543216"), Change::from ("123456"));
        assert_eq! (Change::from ("132546") * Change::from ("123546"), Change::from ("132456"));
    }
    
    #[test]
    #[should_panic]
    fn multiplication_nonequal_stages () {
        let _c = Change::from ("1234") * Change::from ("12345");
    }

    #[test]
    fn inversion () {
        assert_eq! (!Change::from ("12345"), Change::from ("12345"));
        assert_eq! (!Change::from ("1235647890"), Change::from ("1236457890"));
        assert_eq! (!Change::from ("654321"), Change::from ("654321"));
    }
    
    #[test]
    fn debug_print () {
        let mut s = String::with_capacity (20);
        
        write! (&mut s, "{:?}", Change::from ("")).unwrap ();
        assert_eq! (s, "<>");
        s.clear ();
        
        write! (&mut s, "{:?}", Change::from ("14325")).unwrap ();
        assert_eq! (s, "<14325>");
        s.clear ();
        
        write! (&mut s, "{:?}", Change::from ("1678902345ET")).unwrap ();
        assert_eq! (s, "<1678902345ET>");
        s.clear ();
    }
}
