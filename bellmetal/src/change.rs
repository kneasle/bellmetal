use crate::types::*;
use crate::{ Transposition, TranspositionIterator };
use core::ops::{ Mul, Not };
use std::convert::{ From };
use std::fmt;

#[derive(PartialEq, Clone)]
pub struct Change {
    seq : Vec<Bell>
}

impl Transposition for Change {
    fn slice (&self) -> &[Bell] {
        &self.seq [..]
    }
}

impl Change {
    pub fn iterator<'a> (&'a self) -> TranspositionIterator<'a> {
        TranspositionIterator::from_transposition (self)
    }

    pub fn stage (&self) -> Stage {
        Stage::from (self.seq.len ())
    }

    pub fn copy_into (&self, other : &mut Change) {
        other.seq.clear ();
        other.seq.reserve (self.seq.len ());

        for b in &self.seq {
            other.seq.push (*b);
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

    pub fn multiply_iterator<I> (&self, rhs : I) -> Change where I : Iterator<Item = Bell> {
        let mut new_seq : Vec<Bell> = Vec::with_capacity (self.stage ().as_usize ());

        for b in rhs {
            new_seq.push (self.seq [b.as_usize ()]);
        }

        Change { seq : new_seq }
    }
    
    pub fn pre_multiply_into (&self, lhs : &impl Transposition, into : &mut Change) {
        if self.stage () != lhs.stage () {
            panic! ("Can't use transpositions of different stages!");
        }
        
        if self.stage () != into.stage () {
            panic! ("Can't use transpositions of different stages!");
        }

        into.seq.clear ();

        for i in 0..self.stage ().as_usize () {
            into.seq.push (lhs.bell_at (Place::from (self.seq [i].as_number ())));
        }
    }
    
    pub fn multiply_iterator_into<I> (&self, rhs : I, into : &mut Change) where I : Iterator<Item = Bell> {
        if self.stage () != into.stage () {
            panic! ("Can't multiply into a change of the wrong stage");
        }

        into.seq.clear ();
        
        let mut i = 0;

        for b in rhs {
            into.seq.push (self.seq [b.as_usize ()]);

            i += 1;
        }

        assert_eq! (i, into.stage ().as_usize ());
    }
    
    pub fn multiply_into (&self, rhs : &impl Transposition, into : &mut Change) {
        if self.stage () != rhs.stage () {
            panic! ("Can't use transpositions of different stages!");
        }
        
        if self.stage () != into.stage () {
            panic! ("Can't multiply into a change of the wrong stage");
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
        
        if self.stage () != into.stage () {
            panic! ("Can't multiply into a change of the wrong stage");
        }
        
        for i in 0..self.stage ().as_usize () {
            into.seq [rhs.bell_at (Place::from (i)).as_usize ()] = self.seq [i];
        }
    }

    pub fn overwrite_from_string (&mut self, string : &str) {
        self.seq.clear ();
        self.seq.reserve (string.len ());

        for c in string.chars () {
            self.seq.push (Bell::from (c));
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
}

impl Change {
    pub fn empty () -> Change {
        Change {
            seq : Vec::with_capacity (0)
        }
    }

    pub fn rounds (stage : Stage) -> Change {
        let mut seq : Vec<Bell> = Vec::with_capacity (stage.as_usize ());

        for i in 0..stage.as_usize () {
            seq.push (Bell::from (i));
        }

        Change { seq : seq }
    }

    pub fn new (bell_vec : Vec<Bell>) -> Change {
        Change { seq : bell_vec }
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
        let mut change = Change::empty ();

        change.overwrite_from_string (s);

        change
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

    pub fn accumulate_iterator<I> (&mut self, iterator : I) where I : Iterator<Item = Bell> {
        if self.using_second_change {
            self.change_2.multiply_iterator_into (iterator, &mut self.change_1);
        } else {
            self.change_1.multiply_iterator_into (iterator, &mut self.change_2);
        }

        self.using_second_change = !self.using_second_change;
    }

    pub fn accumulate (&mut self, transposition : &impl Transposition) {
        if self.using_second_change {
            self.change_2.multiply_into (transposition, &mut self.change_1);
        } else {
            self.change_1.multiply_into (transposition, &mut self.change_2);
        }

        self.using_second_change = !self.using_second_change;
    }

    pub fn accumulate_inverse (&mut self, transposition : &impl Transposition) {
        if self.using_second_change {
            self.change_2.multiply_inverse_into (transposition, &mut self.change_1);
        } else {
            self.change_1.multiply_inverse_into (transposition, &mut self.change_2);
        }

        self.using_second_change = !self.using_second_change;
    }

    pub fn pre_accumulate (&mut self, iter : &impl Transposition) {
        if self.using_second_change {
            self.change_2.pre_multiply_into (iter, &mut self.change_1);
        } else {
            self.change_1.pre_multiply_into (iter, &mut self.change_2);
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
    use crate::{ 
        Change,
        Bell, Stage, Place, Parity,
        Transposition
    };
    
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
    fn copy_into () {
        let mut change = Change::empty ();
        
        for c in [
            Change::from ("1234"),
            Change::from (""),
            Change::from ("17342685"),
            Change::from ("85672341"),
            Change::from ("0987123456"),
            Change::from ("")
        ].iter () {
            c.copy_into (&mut change);

            assert_eq! (*c, change);
        }
    }

    #[test]
    fn multiplication () {
        let changes = [
            (Change::from ("1324"), Change::from ("4231"), Change::from ("4321")),
            (Change::from ("13425678"), Change::from ("13425678"), Change::from ("14235678")),
            (Change::from ("543216"), Change::from ("543216"), Change::from ("123456")),
            (Change::from ("132546"), Change::from ("123546"), Change::from ("132456"))
        ];

        for (lhs, rhs, result) in &changes {
            assert_eq! (lhs.clone () * rhs.clone (), *result);

            assert_eq! (lhs.multiply_iterator (rhs.iterator ()), *result);
        }
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
    fn iterators () {
        let changes = vec! [
            Change::from ("12345"),
            Change::from ("7298324516"),
            Change::from (""),
            Change::from ("0987123456")
        ];

        for c in changes {
            let mut x = 0;

            for b in c.iterator () {
                assert_eq! (b, c.bell_at (Place::from (x)));

                x += 1;
            }
        }
    }
    
    #[test]
    fn cyclicness_tests () {
        assert! (Change::from ("12345").is_full_cyclic ());
        assert! (Change::from ("5678901234").is_full_cyclic ());
        assert! (!Change::from ("42513").is_full_cyclic ());
        assert! (!Change::from ("14567234").is_full_cyclic ());
        
        assert! (Change::from ("54321").is_reverse_full_cyclic ());
        assert! (Change::from ("7654321098").is_reverse_full_cyclic ());
        assert! (!Change::from ("42513").is_reverse_full_cyclic ());
        assert! (!Change::from ("14567234").is_reverse_full_cyclic ());
        
        assert! (Change::from ("123456789").is_fixed_treble_cyclic ());
        assert! (Change::from ("134562").is_fixed_treble_cyclic ());
        assert! (!Change::from ("4567123").is_fixed_treble_cyclic ());
        assert! (!Change::from ("42513").is_fixed_treble_cyclic ());
        
        assert! (Change::from ("198765432").is_reverse_fixed_treble_cyclic ());
        assert! (Change::from ("126543").is_reverse_fixed_treble_cyclic ());
        assert! (!Change::from ("4567123").is_reverse_fixed_treble_cyclic ());
        assert! (!Change::from ("3217654").is_reverse_fixed_treble_cyclic ());
        assert! (!Change::from ("42513").is_reverse_fixed_treble_cyclic ());
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
