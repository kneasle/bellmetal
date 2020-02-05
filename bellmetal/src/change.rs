use crate::types::*;
use crate::transposition::Transposition;
use core::ops::{ Mul, Not };
use std::convert::{ From };
use std::fmt;

#[derive(PartialEq, Clone)]
pub struct Change {
    pub seq : Vec<Bell>
}

impl Transposition for Change {
    fn stage (&self) -> Stage {
        Stage::from (self.seq.len ())
    }

    fn bell_at (&self, place : Place) -> Bell {
        self.seq [place.as_usize ()]
    }
}

impl Change {
    pub fn stage (&self) -> Stage {
        Stage::from (self.seq.len ())
    }

    pub fn parity (&self) -> Parity {
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

    pub fn multiply (&self, rhs : &impl Transposition) -> Change {
        if self.stage () != rhs.stage () {
            panic! ("Can't use transpositions of different stages!");
        }

        let mut new_seq : Vec<Bell> = Vec::with_capacity (self.stage ().as_usize ());

        for i in 0..self.stage ().as_usize () {
            new_seq.push (self.seq [rhs.bell_at (Place::from (i)).as_usize ()]);
        }

        Change { seq : new_seq }
    }
    
    pub fn pre_multiply_into (&self, lhs : &impl Transposition, into : &mut Change) {
        if self.stage () != lhs.stage () {
            panic! ("Can't use transpositions of different stages!");
        }

        into.seq.clear ();

        for i in 0..self.stage ().as_usize () {
            into.seq.push (lhs.bell_at (Place::from (self.seq [i].as_number ())));
        }
    }
    
    pub fn multiply_into (&self, rhs : &impl Transposition, into : &mut Change) {
        if self.stage () != rhs.stage () {
            panic! ("Can't use transpositions of different stages!");
        }

        into.seq.clear ();

        for i in 0..self.stage ().as_usize () {
            into.seq.push (self.seq [rhs.bell_at (Place::from (i)).as_usize ()]);
        }
    }

    pub fn multiply_inverse_into (&self, rhs : &impl Transposition, into : &mut Change) {
        if self.stage () != rhs.stage () {
            panic! ("Can't use transpositions of different stages!");
        }
        
        for i in 0..self.stage ().as_usize () {
            into.seq [rhs.bell_at (Place::from (i)).as_usize ()] = self.seq [i];
        }
    }

    pub fn inverse (&self) -> Change {
        let mut new_seq : Vec<Bell> = vec![Bell::from (0u32); self.stage ().as_usize ()];

        for i in 0..self.stage ().as_usize () {
            new_seq [self.seq [i as usize].as_usize ()] = Bell::from (i);
        }

        Change { seq : new_seq }
    }

    pub fn pow (&self, exponent : i32) -> Change {
        if exponent == 0 {
            return Change::rounds (self.stage ());
        }

        let mut accumulator = ChangeAccumulator::new (self.stage ());
        
        if exponent > 0 {
            for _ in 0..exponent as usize {
                accumulator.accumulate (self);
            }
        } else {
            for _ in 0..(-exponent) as usize {
                accumulator.accumulate_inverse (self);
            }
        }

        accumulator.total ().clone ()
    }

    pub fn is_full_cyclic (&self) -> bool {
        let stage = self.stage ().as_usize ();

        if stage == 0 {
            return false;
        }

        let start = self.seq [0].as_usize ();

        for i in 0..stage {
            if self.seq [i].as_usize () != (start + i) % stage {
                return false;
            }
        }

        true
    }

    pub fn is_fixed_treble_cyclic (&self) -> bool {
        let stage = self.stage ().as_usize ();
        
        if stage <= 2 || self.seq [0].as_usize () != 0 {
            return false;
        }

        let start = self.seq [1].as_usize ();

        for i in 0..stage - 1 {
            let expected_bell = if start + i >= stage { start + i - stage + 1 } else { start + i };
            
            if self.seq [i + 1].as_usize () != expected_bell {
                return false;
            }
        }

        true
    }

    pub fn is_backrounds (&self) -> bool {
        let stage = self.stage ().as_usize ();

        for i in 0..stage {
            if self.seq [i].as_usize () != stage - 1 - i {
                return false;
            }
        }

        true
    }

    pub fn is_rounds (&self) -> bool {
        for i in 0..self.stage ().as_usize () {
            if self.seq [i].as_usize () != i {
                return false;
            }
        }

        true
    }
    
    // Music scoring (follows roughly what CompLib does)
    pub fn music_score (&self) -> usize {
        Change::run_length_to_score (self.run_length_off_front ()) 
            + Change::run_length_to_score (self.run_length_off_back ())
    }

    fn run_length_to_score (length : usize) -> usize {
        if length < 4 {
            return 0;
        }

        let x = length - 3;
        
        // Triangular numbers = n * (n + 1) / 2
        (x * (x + 1)) >> 1
    }

    fn run_length_off_front (&self) -> usize {
        let stage = self.stage ().as_usize ();

        if stage <= 1 {
            return stage;
        }

        let mut last = self.seq [0].as_number ();
        let mut i = 1;

        while i < stage && (
            self.seq [i].as_i32 () - last as i32 == -1 
         || self.seq [i].as_i32 () - last as i32 == 1
        ) {
            last = self.seq [i].as_number ();

            i += 1;
        }
        
        i
    }
    
    fn run_length_off_back (&self) -> usize {
        let stage = self.stage ().as_usize ();

        if stage <= 1 {
            return stage;
        }

        let mut last = self.seq [stage - 1].as_number ();
        let mut i = 1;

        while i < stage && (
            self.seq [stage - 1 - i].as_i32 () - last as i32 == -1 
         || self.seq [stage - 1 - i].as_i32 () - last as i32 == 1
        ) {
            last = self.seq [stage - 1 - i].as_number ();

            i += 1;
        }
        
        i
    }

    // Pretty printing
    pub fn pretty_string (&self) -> String {
        let stage = self.stage ().as_usize ();

        let mut string = String::with_capacity (stage * 3); // Seems a good length

        let run_front = {
            let x = self.run_length_off_front ();

            if x >= 4 { x } else { 0 }
        };

        let run_back = {
            let x = self.run_length_off_back ();

            if x >= 4 { x } else { 0 }
        };

        let mut was_last_char_highlighted = false;
        let mut last_char_colour = 0;

        let colours = ["0", "91", "96"];

        for i in 0..stage {
            // Useful vars
            let bell = self.seq [i];

            let char_colour = if bell.as_usize () == 0 {
                1
            } else if bell.as_usize () == stage - 1 {
                2
            } else {
                0
            };
            
            let should_be_highlighted = i < run_front || (stage - 1 - i) < run_back;
            
            // Push the escape codes
            if last_char_colour != char_colour || was_last_char_highlighted != should_be_highlighted {
                string.push_str ("\x1b[");
                string.push_str (colours [char_colour]);
                string.push (';');
                string.push_str (if should_be_highlighted { "42" } else { "49" });
                string.push ('m');
            }
            
            string.push (bell.as_char ());

            was_last_char_highlighted = should_be_highlighted;
            last_char_colour = char_colour;
        }

        string.push_str ("\x1b[0m"); // Always reset the formatting

        string
    }
    
    // "Static" methods
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

    fn mul (self, rhs : Change) -> Self {
        self.multiply (&rhs)
    }
}

impl Not for Change {
    type Output = Self;

    fn not (self) -> Self {
        self.inverse ()
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








pub struct ChangeAccumulator {
    change_1 : Change,
    change_2 : Change,
    stage : Stage,
    using_second_change : bool
}

impl ChangeAccumulator {
    pub fn new (stage : Stage) -> ChangeAccumulator {
        ChangeAccumulator {
            change_1 : Change::rounds (stage),
            change_2 : Change::rounds (stage),
            stage : stage,
            using_second_change : false
        }
    }

    pub fn total (&self) -> &Change {
        if self.using_second_change {
            &(self.change_2)
        } else {
            &(self.change_1)
        }
    }

    pub fn accumulate (&mut self, transposition : &impl Transposition) {
        if self.using_second_change {
            self.change_2.multiply_into (transposition, &mut self.change_1)
        } else {
            self.change_1.multiply_into (transposition, &mut self.change_2)
        }

        self.using_second_change = !self.using_second_change;
    }

    pub fn accumulate_inverse (&mut self, transposition : &impl Transposition) {
        if self.using_second_change {
            self.change_2.multiply_inverse_into (transposition, &mut self.change_1)
        } else {
            self.change_1.multiply_inverse_into (transposition, &mut self.change_2)
        }

        self.using_second_change = !self.using_second_change;
    }

    pub fn pre_accumulate (&mut self, iter : &impl Transposition) {
        if self.using_second_change {
            self.change_2.pre_multiply_into (iter, &mut self.change_1)
        } else {
            self.change_1.pre_multiply_into (iter, &mut self.change_2)
        }
    }

    pub fn set (&mut self, change : &Change) {
        if self.stage != change.stage () {
            panic! ("Can't write a change of the wrong stage into accumulator");
        }
        
        if self.using_second_change {
            for i in 0..self.stage.as_usize () {
                self.change_2.seq [i] = change.seq [i];
            }
        } else {
            for i in 0..self.stage.as_usize () {
                self.change_1.seq [i] = change.seq [i];
            }
        }
    }

    pub fn reset (&mut self) {
        for i in 0..self.stage.as_usize () {
            self.change_1.seq [i] = Bell::from (i);
            self.change_2.seq [i] = Bell::from (i);
            
            self.using_second_change = false;
        }
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
    fn multiply_into () {
        let mut change = Change::rounds (Stage::MAJOR);

        Change::from ("15678234").multiply_into (&Change::from ("13456782"), &mut change);
        assert_eq! (change, Change::from ("16782345"));

        Change::from ("15678234").multiply_into (&Change::from ("87654321"), &mut change);
        assert_eq! (change, Change::from ("43287651"));
    }

    #[test]
    fn multiply_inverse_into () {
        let mut change = Change::rounds (Stage::MAJOR);

        Change::from ("15678234").multiply_inverse_into (&Change::from ("18234567"), &mut change);
        assert_eq! (change, Change::from ("16782345"));

        Change::from ("15678234").multiply_inverse_into (&Change::from ("87654321"), &mut change);
        assert_eq! (change, Change::from ("43287651"));
    }

    #[test]
    fn exponentiation () {
        assert_eq! (Change::from ("18765432").pow (2i32), Change::rounds (Stage::from (8)));
        assert_eq! (Change::from ("912345678").pow (-4), Change::from ("567891234"));
        assert_eq! (Change::from ("134265").pow (2), Change::from ("142356"));
        assert_eq! (Change::from ("134265").pow (-3), Change::from ("123465"));
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
    fn cyclicness_tests () {
        assert! (Change::from ("12345").is_full_cyclic ());
        assert! (Change::from ("5678901234").is_full_cyclic ());
        assert! (!Change::from ("42513").is_full_cyclic ());
        assert! (!Change::from ("14567234").is_full_cyclic ());

        assert! (Change::from ("123456789").is_fixed_treble_cyclic ());
        assert! (Change::from ("134562").is_fixed_treble_cyclic ());
        assert! (!Change::from ("4567123").is_fixed_treble_cyclic ());
        assert! (!Change::from ("42513").is_fixed_treble_cyclic ());
    }

    #[test]
    fn backrounds_test () {
        assert! (Change::from ("4321").is_backrounds ());
        assert! (Change::from ("1").is_backrounds ());
        assert! (Change::from ("").is_backrounds ());
        assert! (!Change::from ("7584012369").is_backrounds ());
        assert! (!Change::from ("4567123").is_backrounds ());
    }

    #[test]
    fn rounds_test () {
        assert! (Change::from ("1234567890E").is_rounds ());
        assert! (Change::from ("1").is_rounds ());
        assert! (Change::from ("").is_rounds ());
        assert! (!Change::from ("7584012369").is_rounds ());
        assert! (!Change::from ("4567123").is_rounds ());
    }

    #[test]
    fn music_run_lengths () {
        assert_eq! (Change::from ("14238765").run_length_off_front (), 1);
        assert_eq! (Change::from ("12346578").run_length_off_front (), 4);
        assert_eq! (Change::from ("12345678").run_length_off_front (), 8);
        assert_eq! (Change::from ("76543218").run_length_off_front (), 7);
        
        assert_eq! (Change::from ("81765432").run_length_off_back (), 6);
        assert_eq! (Change::from ("14238765").run_length_off_back (), 4);
        assert_eq! (Change::from ("76543218").run_length_off_back (), 1);
        assert_eq! (Change::from ("1234567890").run_length_off_back (), 10);
    }

    #[test]
    fn music_scoring () {
        assert_eq! (Change::from ("12347568").music_score (), 1);
        assert_eq! (Change::from ("567894231").music_score (), 3);
        assert_eq! (Change::from ("1234908765").music_score (), 2);
        assert_eq! (Change::from ("1234560978").music_score (), 6);
        assert_eq! (Change::from ("1234560987").music_score (), 7);
        assert_eq! (Change::from ("1234").music_score (), 2);
        assert_eq! (Change::from ("15234").music_score (), 0);
        assert_eq! (Change::from ("9876543210").music_score (), 21);
        assert_eq! (Change::from ("0987654321").music_score (), 56);
    }

    /* #[test]
    fn pretty_print () {
        println! ("{}", Change::from ("12346578").pretty_string ());
        println! ("{}", Change::from ("87654312TE09").pretty_string ());
        println! ("{}", Change::from ("13245678").pretty_string ());

        assert! (false);
    } */
    
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
