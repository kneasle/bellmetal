use crate::{ Touch, Row, Stage, Place, Bell, Transposition, Change };

pub trait ProvingContext {
    fn prove_canonical (&mut self, touch : &Touch, canon : impl FnMut(&Row, &mut Change) -> ()) -> bool;
    fn prove (&mut self, touch : &Touch) -> bool;
}






pub struct NaiveProver { }

impl ProvingContext for NaiveProver {
    fn prove_canonical (&mut self, touch : &Touch, mut canon : impl FnMut(&Row, &mut Change) -> ()) -> bool {
        let mut temporary_change = Change::rounds (touch.stage);

        let mut changes : Vec<Change> = Vec::with_capacity (touch.length);

        for row in touch.row_iterator () {
            canon (&row, &mut temporary_change);

            changes.push (temporary_change.clone ());
        }

        changes.sort ();

        for i in 1..changes.len () {
            if changes [i - 1] == changes [i] {
                return false;
            }
        }

        true
    }

    fn prove (&mut self, touch : &Touch) -> bool {
        let mut rows : Vec<Row> = touch.row_iterator ().collect ();

        rows.sort ();

        for i in 1..rows.len () {
            if rows [i - 1] == rows [i] {
                return false;
            }
        }

        true
    }
}







struct BitMap {
    vec : Vec<u64>
}

impl BitMap {
    pub fn set (&mut self, index : usize, val : bool) {
        if val {
            self.set_true (index);
        } else {
            self.set_false (index);
        }
    }

    pub fn set_false (&mut self, index : usize) {
        self.vec [index >> 6] &= !(1 << (index & 0b11_1111));
    }

    pub fn set_true (&mut self, index : usize) {
        self.vec [index >> 6] |= 1 << (index & 0b11_1111)
    }

    pub fn get (&self, index : usize) -> bool {
        self.vec [index >> 6] & (1 << (index & 0b11_1111)) != 0
    }

    pub fn clear (&mut self) {
        for i in 0..self.vec.len () {
            self.vec [i] = 0;
        }
    }
}

impl BitMap {
    pub fn with_capacity (size : usize) -> BitMap {
        BitMap {
            vec : vec![0; (size >> 6) + 1]
        }
    }
}







pub struct HashProver {
    stage : Stage,
    bit_map : BitMap
}

impl ProvingContext for HashProver {
    fn prove_canonical (&mut self, touch : &Touch, mut canon : impl FnMut(&Row, &mut Change) -> ()) -> bool {
        assert_eq! (touch.stage, self.stage);

        let mut truth = true;
        let mut temporary_change = Change::rounds (touch.stage);

        for r in touch.row_iterator () {
            canon (&r, &mut temporary_change);
            
            let hash = temporary_change.naive_hash ();
            
            if self.bit_map.get (hash) {
                truth = false;
                break;
            }

            self.bit_map.set_true (hash);
        }

        // Reset the hash map before returning
        for r in touch.row_iterator () {
            self.bit_map.set_false (r.naive_hash ());
        }

        truth
    }

    fn prove (&mut self, touch : &Touch) -> bool {
        assert_eq! (touch.stage, self.stage);

        let mut truth = true;

        for r in touch.row_iterator () {
            let hash = r.naive_hash ();
            
            if self.bit_map.get (hash) {
                truth = false;
                break;
            }

            self.bit_map.set_true (hash);
        }

        // Reset the hash map before returning
        for r in touch.row_iterator () {
            self.bit_map.set_false (r.naive_hash ());
        }

        truth
    }
}

impl HashProver {
    pub fn from_stage (stage : Stage) -> HashProver {
        let s = stage.as_usize ();

        assert! (s <= 8);

        HashProver {
            stage : stage,
            bit_map : BitMap::with_capacity (s.pow (s as u32))
        }
    }
}






#[cfg(test)]
mod bitmap_tests {
    use crate::proving::*;

    #[test]
    fn basic () {
        let mut map = BitMap::with_capacity (500);

        map.set_true (0);
        map.set (21, true);

        for i in 400..450 {
            map.set (i, true);

            assert_eq! (map.get (i), true);

            map.set_false (i);

            assert_eq! (map.get (i), false);

            map.set_true (i);

            assert_eq! (map.get (i), true);

            assert_eq! (map.get (0), true);
        }

        assert_eq! (map.get (0), true);
        assert_eq! (map.get (10), false);

        map.set_false (0);
        
        assert_eq! (map.get (0), false);

        map.clear ();

        for i in 0..500 {
            assert_eq! (map.get (i), false);
        }
    }
}






// Example canonical functions
pub fn canon_copy (row : &Row, change : &mut Change) {
    row.copy_into (change);
}

pub fn canon_fixed_treble_cyclic (row : &Row, change : &mut Change) {
    // We'll convert so that the first non-treble bell in the change is the 2
    let slice = row.slice ();
    let stage = row.stage ().as_usize ();

    // Nothing to be done if the stage is one
    if stage == 1 {
        return;
    }

    if slice [0] == Bell::from (0) {
        let shift = slice [1].as_isize () - 1;
                
        change.set_bell (Place::from (0), Bell::from (0));

        for i in 2..stage {
            let new_bell = slice [i].as_isize () - shift;

            if new_bell <= 0 {
                change.set_bell (Place::from (i), Bell::from ((stage as isize - 1 + new_bell) as usize));
            } else {
                change.set_bell (Place::from (i), Bell::from (new_bell as usize));
            }
        }
    } else {
        let shift = slice [0].as_isize () - 1;
                
        for i in 0..stage {
            if slice [i] == Bell::from (0) {
                change.set_bell (Place::from (i), Bell::from (0));
                continue;
            }

            let new_bell = slice [i].as_isize () - shift;

            if new_bell <= 0 {
                change.set_bell (Place::from (i), Bell::from ((stage as isize - 1 + new_bell) as usize));
            } else {
                change.set_bell (Place::from (i), Bell::from (new_bell as usize));
            }
        }
    }
}

pub fn canon_full_cyclic (row : &Row, change : &mut Change) {
    // We'll convert so that the first non-treble bell in the change is the 2
    let slice = row.slice ();
    let stage = row.stage ().as_usize ();

    // Nothing to be done if the stage is one
    if stage == 1 {
        return;
    }

    let shift = slice [0].as_usize ();
            
    for i in 0..stage {
        let new_bell = slice [i].as_usize () + stage - shift;

        change.set_bell (Place::from (i), Bell::from (new_bell % stage));
    }
}






#[cfg(test)]
mod proof_tests {
    use crate::{ Touch };
    use crate::proving::*;

    fn test_touches () -> Vec<(Touch, bool)> {
        vec![
            (Touch::from ("123"), true),
            (Touch::from ("123456\n214365\n123456"), true),
            (Touch::from ("123456\n214365\n123456\n123456"), false),
        ]
    }
    
    #[test]
    fn naive () {
        for (t, b) in test_touches () {
            assert_eq! (NaiveProver { }.prove_canonical (&t, canon_copy), b);
            assert_eq! (NaiveProver { }.prove (&t), b);
        }
    }
    
    #[test]
    fn hash () {
        for (t, b) in test_touches () {
            if t.stage.as_usize () <= 8 {
                assert_eq! (HashProver::from_stage (t.stage).prove_canonical (&t, canon_copy), b);
                assert_eq! (HashProver::from_stage (t.stage).prove (&t), b);
            }
        }
    }

    #[test]
    fn canon_func_fixed_treble_cyclic () {
        for (orig, canon) in &[
            ("1\n1", "1"),
            ("132\n123", "123"),
            ("123456\n123456", "123456"),
            ("42315678\n12345678", "27813456"),
            ("71632548\n12345678", "21854763"),
            ("87654321\n12345678", "28765431"),
            ("4567890231\n1234567890", "2345678901")
        ] {
            let touch = Touch::from (*orig);
            let mut change = Change::rounds (touch.stage);

            canon_fixed_treble_cyclic (&touch.row_iterator ().next ().unwrap (), &mut change);

            assert_eq! (change, Change::from (*canon));
        }
    }

    #[test]
    fn canon_func_full_cyclic () {
        for (orig, canon) in &[
            ("1\n1", "1"),
            ("132\n123", "132"),
            ("123456\n123456", "123456"),
            ("42315678\n12345678", "17862345"),
            ("71632548\n12345678", "13854762"),
            ("87654321\n12345678", "18765432"),
            ("4567890231\n1234567890", "1234567908")
        ] {
            let touch = Touch::from (*orig);
            let mut change = Change::rounds (touch.stage);

            canon_full_cyclic (&touch.row_iterator ().next ().unwrap (), &mut change);

            assert_eq! (change, Change::from (*canon));
        }
    }

    #[test]
    fn canonical_proving () {
        for (touch, truth) in &[
            ("123\n132\n123", false),
            ("12345678\n21436587\n17654328\n31547682\n12345678", true),
            ("12345678\n21436587\n17654328\n31547628\n12345678", false),
        ] {
            let t = Touch::from (*touch);

            assert_eq! (
                HashProver::from_stage (t.stage).prove_canonical (&t, canon_fixed_treble_cyclic), 
                *truth
            );
        }
    }
}
