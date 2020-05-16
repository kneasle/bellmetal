use crate::types::*;
use crate::{ Transposition };
use core::ops::{ Mul, Not };
use std::convert::{ From };
use std::fmt;

#[derive(Hash, PartialOrd, Ord, Eq, PartialEq, Clone)]
pub struct Change {
    seq : Vec<Bell>
}

impl Transposition for Change {
    fn slice (&self) -> &[Bell] {
        &self.seq
    }
}

impl Change {
    pub fn stage (&self) -> Stage {
        Stage::from (self.seq.len ())
    }

    pub fn mut_slice (&mut self) -> &mut [Bell] {
        &mut self.seq
    }

    pub fn multiply (&self, rhs : &impl Transposition) -> Change {
        let mut c = Change::empty ();

        self.multiply_into (rhs, &mut c);

        c
    }

    pub fn set_bell (&mut self, place : Place, bell : Bell) {
        self.seq [place.as_usize ()] = bell;
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

        into.seq.clear ();

        for i in 0..self.stage ().as_usize () {
            into.seq.push (self.seq [rhs.bell_at (Place::from (i)).as_usize ()]);
        }
    }

    pub fn multiply_inverse (&self, rhs : &impl Transposition) -> Change {
        let mut change = Change::rounds (self.stage ());

        self.multiply_inverse_into (rhs, &mut change);

        change
    }

    pub fn multiply_inverse_into (&self, rhs : &impl Transposition, into : &mut Change) {
        if self.stage () != rhs.stage () {
            panic! ("Can't use transpositions of different stages!");
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

    pub fn overwrite_from_iterator<T> (&mut self, iter : T)
        where T : Iterator<Item = Bell>, T : Sized
    {
        self.seq.clear ();
        self.seq.extend (iter);
    }

    pub fn overwrite_from_slice (&mut self, slice : &[Bell]) {
        self.seq.clear ();
        self.seq.reserve (slice.len ());

        for b in slice.iter () {
            self.seq.push (*b);
        }
    }

    pub fn overwrite_from (&mut self, other : &impl Transposition) {
        self.overwrite_from_slice (other.slice ());
    }

    pub fn inverse (&self) -> Change {
        let mut new_seq : Vec<Bell> = vec![Bell::from (0 as Number); self.stage ().as_usize ()];

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

    pub fn in_place_fixed_treble_inverse (&mut self) {
        let stage = self.seq.len ();

        for i in 0..stage {
            if self.seq [i] != Bell::from (0) {
                self.seq [i] = Bell::from (stage - self.seq [i].as_usize ());
            }
        }
    }

    pub fn in_place_inverse (&mut self) {
        let stage = self.seq.len ();

        for i in 0..stage {
            self.seq [i] = Bell::from (stage - 1 - self.seq [i].as_usize ());
        }
    }

    pub fn in_place_full_cyclic_rotate (&mut self, amount : usize) {
        let stage = self.seq.len ();

        for i in 0..stage {
            self.seq [i] = Bell::from ((self.seq [i].as_usize () + amount) % stage);
        }
    }

    pub fn in_place_fixed_treble_cyclic_rotate (&mut self, amount : usize) {
        let stage = self.seq.len ();

        for i in 0..stage {
            if self.seq [i] != Bell::from (0) {
                let new_bell = self.seq [i].as_usize () + amount;

                if new_bell >= stage {
                    self.seq [i] = Bell::from (new_bell + 1 - stage);
                } else {
                    self.seq [i] = Bell::from (new_bell);
                }
            }
        }
    }

    pub fn in_place_reverse (&mut self) {
        let stage = self.seq.len ();

        for i in 0..stage / 2 {
            let tmp = self.seq [i];
            self.seq [i] = self.seq [stage - 1 - i];
            self.seq [stage - 1 - i] = tmp;
        }
    }

    pub fn destructive_hash (&mut self) -> usize {
        let stage = self.seq.len ();

        let mut hash = 0;
        let mut multiplier = 1;

        for i in 1..stage {
            multiplier *= i;
        }

        let mut i = stage - 1;

        while i > 0 {
            for j in 0..stage {
                if self.seq [j] == Bell::from (i) {
                    hash += j * multiplier;

                    self.seq [j] = self.seq [i];

                    break;
                }
            }

            multiplier /= i;
            i -= 1;
        }

        hash
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

    pub fn from_iterator<T> (iter : T) -> Change
        where T : Iterator<Item = Bell>, T : Sized
    {
        let mut c = Change::empty ();

        c.overwrite_from_iterator (iter);

        c
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

        self.using_second_change = !self.using_second_change;
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

impl ChangeAccumulator {
    pub fn new (stage : Stage) -> ChangeAccumulator {
        ChangeAccumulator {
            change_1 : Change::rounds (stage),
            change_2 : Change::rounds (stage),
            stage : stage,
            using_second_change : false
        }
    }
}





pub struct ChangeCollectIter<T : Iterator<Item = Bell>> {
    bell_iter : T,
    stage : Stage
}

impl<T : Iterator<Item = Bell>> ChangeCollectIter<T> {
    pub fn new (bell_iter : T, stage : Stage) -> ChangeCollectIter<T> {
        ChangeCollectIter {
            bell_iter : bell_iter,
            stage : stage
        }
    }
}

impl<T : Iterator<Item = Bell>> Iterator for ChangeCollectIter<T> {
    type Item = Change;

    fn next (&mut self) -> Option<Change> {
        let stage = self.stage.as_usize ();
        let mut vec : Vec<Bell> = Vec::with_capacity (stage);

        for _ in 0..stage {
            if let Some (b) = self.bell_iter.next () {
                vec.push (b);
            } else {
                return None;
            }
        }

        Some (Change::new (vec))
    }
}






#[cfg(test)]
mod change_tests {
    use crate::{
        Change,
        Bell, Stage, Place, Parity,
        Transposition
    };

    use crate::utils::ExtentIterator;

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

        for c in &[
            Change::from ("1234"),
            Change::from (""),
            Change::from ("17342685"),
            Change::from ("85672341"),
            Change::from ("0987123456"),
            Change::from ("")
        ] {
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

            assert_eq! (lhs.multiply_iterator (rhs.iter ()), *result);
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
        assert_eq! (Change::from ("81275643").pow (0), Change::rounds (Stage::from (8)));
        assert_eq! (Change::from ("912345678").pow (-4), Change::from ("567891234"));
        assert_eq! (Change::from ("134265").pow (2), Change::from ("142356"));
        assert_eq! (Change::from ("134265").pow (-3), Change::from ("123465"));
    }

    #[test]
    #[should_panic]
    fn multiplication_nonequal_stages () {
        let _ = Change::from ("1234") * Change::from ("12345");
    }

    #[test]
    #[should_panic]
    fn multiplicaty_invert_nonequal_stages () {
        Change::from ("1234").multiply_inverse_into (&Change::from ("12345"), &mut Change::empty ());
    }

    #[test]
    #[should_panic]
    fn pre_multiplication_into_nonequal_stages_1 () {
        Change::from ("1234").pre_multiply_into (&Change::from ("12345"), &mut Change::empty ());
    }

    #[test]
    fn pre_multiplication_into_nonequal_stages_2 () {
        let mut c = Change::empty ();

        Change::from ("32145678").pre_multiply_into (&Change::from ("78123456"), &mut c);

        assert_eq! (c, Change::from ("18723456"));
    }

    #[test]
    fn mut_slice () {
        let mut c = Change::rounds (Stage::MAXIMUS);

        c.mut_slice () [5] = Bell::from (0);

        assert_eq! (c, Change::from ("1234517890ET"));
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

            for b in c.iter () {
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
    fn in_place_inverse () {
        for (from, to) in &[
            ("1", "1"),
            ("4231", "1324"),
            ("14235", "52431"),
            ("12345678", "87654321")
        ] {
            let mut change = Change::from (*from);

            change.in_place_inverse ();

            assert_eq! (change, Change::from (*to));
        }
    }

    #[test]
    fn in_place_fixed_treble_cyclic_rotate () {
        for (from, amount, to) in &[
            ("1", 5, "1"),
            ("12345", 3, "15234"),
            ("43215678", 6, "32814567"),
            ("1425367890", 5, "1970823456")
        ] {
            let mut change = Change::from (*from);

            change.in_place_fixed_treble_cyclic_rotate (*amount);

            assert_eq! (change, Change::from (*to));
        }
    }

    #[test]
    fn in_place_full_cyclic_rotate () {
        for (from, amount, to) in &[
            ("1", 5, "1"),
            ("12345", 3, "45123"),
            ("43215678", 6, "21873456"),
            ("1425367890", 5, "6970812345")
        ] {
            let mut change = Change::from (*from);

            change.in_place_full_cyclic_rotate (*amount);

            assert_eq! (change, Change::from (*to));
        }
    }

    #[test]
    fn in_place_fixed_treble_inverse () {
        for (from, to) in &[
            ("1", "1"),
            ("4231", "2431"),
            ("14235", "13542"),
            ("12345678", "18765432")
        ] {
            let mut change = Change::from (*from);

            change.in_place_fixed_treble_inverse ();

            assert_eq! (change, Change::from (*to));
        }
    }

    #[test]
    fn in_place_reverse () {
        for s in &[
            "1",
            "4231",
            "14235",
            "12345678",
        ] {
            let mut change = Change::from (*s);

            change.in_place_reverse ();

            assert_eq! (change, Change::from (&s.chars ().rev ().collect::<String> () [..]));
        }
    }

    #[test]
    fn destructive_hash () {
        for s in 1..9 {
            let mut hashes : Vec<usize> = ExtentIterator::new (Stage::from (s))
                .map (|mut x| x.destructive_hash ())
                .collect ();

            hashes.sort ();

            for s in 0..hashes.len () {
                assert_eq! (s, hashes [s]);
            }
        }
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

#[cfg(test)]
mod change_accum_tests {
    use crate::{ Stage, Change, ChangeAccumulator };

    #[test]
    #[should_panic]
    fn set_wrong_stage () {
        let mut acc = ChangeAccumulator::new (Stage::MAJOR);

        acc.set (&Change::from ("123"));
    }

    #[test]
    fn non_panicking_behaviour () {
        let mut acc = ChangeAccumulator::new (Stage::MAJOR);

        assert_eq! (acc.total (), &Change::rounds (Stage::MAJOR));

        acc.accumulate (&Change::from ("43215678"));
        assert_eq! (acc.total (), &Change::from ("43215678"));
        acc.accumulate (&Change::from ("43215678"));
        assert_eq! (acc.total (), &Change::from ("12345678"));

        acc.accumulate (&Change::from ("34567812"));
        assert_eq! (acc.total (), &Change::from ("34567812"));
        acc.accumulate (&Change::from ("34567812"));
        assert_eq! (acc.total (), &Change::from ("56781234"));

        acc.accumulate_inverse (&Change::from ("31245678"));
        assert_eq! (acc.total (), &Change::from ("67581234"));

        acc.pre_accumulate (&Change::from ("81234567"));
        assert_eq! (acc.total (), &Change::from ("56478123"));
        acc.pre_accumulate (&Change::from ("45678123"));
        assert_eq! (acc.total (), &Change::from ("81723456"));

        acc.set (&Change::from ("74651238"));
        assert_eq! (acc.total (), &Change::from ("74651238"));

        acc.pre_accumulate (&Change::from ("45678123"));
        assert_eq! (acc.total (), &Change::from ("27184563"));

        acc.set (&Change::from ("74865123"));
        assert_eq! (acc.total (), &Change::from ("74865123"));

        acc.reset ();
        assert_eq! (acc.total (), &Change::rounds (Stage::MAJOR));
    }
}

#[cfg(test)]
mod change_collection_tests {
    use crate::{ Stage, Change, ChangeCollectIter, Transposition };

    #[test]
    fn normal_usage () {
        let c = Change::from ("123452143524153425134523154321");

        let mut iter = ChangeCollectIter::new (
            c.iter (),
            Stage::DOUBLES
        );

        assert_eq! (iter.next (), Some (Change::from ("12345")));
        assert_eq! (iter.next (), Some (Change::from ("21435")));
        assert_eq! (iter.next (), Some (Change::from ("24153")));
        assert_eq! (iter.next (), Some (Change::from ("42513")));
        assert_eq! (iter.next (), Some (Change::from ("45231")));
        assert_eq! (iter.next (), Some (Change::from ("54321")));
        assert_eq! (iter.next (), None);
    }

    #[test]
    fn incorrect_length () {
        let c = Change::from ("1234521435241534251345231543214523");

        let mut iter = ChangeCollectIter::new (
            c.iter (),
            Stage::DOUBLES
        );

        assert_eq! (iter.next (), Some (Change::from ("12345")));
        assert_eq! (iter.next (), Some (Change::from ("21435")));
        assert_eq! (iter.next (), Some (Change::from ("24153")));
        assert_eq! (iter.next (), Some (Change::from ("42513")));
        assert_eq! (iter.next (), Some (Change::from ("45231")));
        assert_eq! (iter.next (), Some (Change::from ("54321")));
        assert_eq! (iter.next (), None);
    }
}
