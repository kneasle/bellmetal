use crate::{ Touch, Row, Stage, Transposition };

pub trait ProvingContext {
    fn prove (&mut self, touch : &Touch) -> bool;
}






pub struct NaiveProver { }

impl ProvingContext for NaiveProver {
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
            assert_eq! (NaiveProver { }.prove (&t), b);
        }
    }
    
    #[test]
    fn hash () {
        for (t, b) in test_touches () {
            if t.stage.as_usize () <= 8 {
                assert_eq! (HashProver::from_stage (t.stage).prove (&t), b);
            }
        }
    }
}
