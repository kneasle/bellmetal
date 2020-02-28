use crate::{ Change, Stage, Bell };

pub fn closure (change : Change) -> Vec<Change> {
    let mut vec : Vec<Change> = Vec::with_capacity (change.stage ().as_usize ());

    let rounds = Change::rounds (change.stage ());
    let mut accum = change.clone ();

    vec.push (rounds.clone ());

    while accum != rounds {
        vec.push (accum.clone ());

        accum = accum * change.clone ();
    }

    vec
}





pub struct ExtentIterator {
    generator : ExtentGenerator,
    is_done : bool
}

impl Iterator for ExtentIterator {
    type Item = Change;

    fn next (&mut self) -> Option<Change> {
        if self.is_done {
            None
        } else {
            let stage = self.generator.stage.as_usize ();

            let mut bell_vec : Vec<Bell> = Vec::with_capacity (stage);

            for _ in 0..stage {
                bell_vec.push (Bell::from (0));
            }

            self.generator.fill (&mut bell_vec [..]);

            self.is_done = !self.generator.step ();

            Some (Change::new (bell_vec))
        }
    }
}

impl ExtentIterator {
    pub fn new (stage : Stage) -> ExtentIterator {
        ExtentIterator {
            generator : ExtentGenerator::new (stage),
            is_done : stage == Stage::ZERO
        }
    }
}





struct ExtentGenerator {
    recursive_generator : Option<Box<ExtentGenerator>>,
    pub stage : Stage,
    insert_location : usize,
    array_index : usize
}

impl ExtentGenerator {
    // Moves the iterator on by a step, returns true if there is are more permutations to come
    pub fn step (&mut self) -> bool {
        let stage = self.stage.as_usize ();
        
        if stage < CACHED_EXTENTS {
            self.array_index += 1;

            return self.array_index < EXTENT_LENGTHS [stage];
        } else {
            match &mut self.recursive_generator {
                Some (g) => {
                    self.insert_location += 1;

                    if self.insert_location == stage {
                        self.insert_location = 0;

                        return g.step ();
                    } else {
                        return true;
                    }
                }
                None => {
                    panic! ("Recursive extent generator not found");
                }
            }
        }
    }
    
    pub fn fill (&self, slice : &mut [Bell]) {
        let stage = self.stage.as_usize ();

        if stage < CACHED_EXTENTS {
            // Load the extent from static memory
            let extent_slice = EXTENTS [stage];

            for i in 0..stage {
                slice [i] = Bell::from (extent_slice [self.array_index * stage + i] as usize);
            }
        } else {
            // Generate the permutation recursively
            match &self.recursive_generator {
                Some (g) => {
                    g.fill (slice);
                    
                    let mut temp = Bell::from (stage - 1);

                    for i in self.insert_location..stage {
                        let t2 = slice [i];
                        slice [i] = temp;
                        temp = t2;
                    }
                }
                None => { 
                    panic! ("Recursive extent generator not found"); 
                }
            }
        }
    }
}

impl ExtentGenerator {
    pub fn new (stage : Stage) -> ExtentGenerator {
        let s = stage.as_usize ();

        ExtentGenerator {
            recursive_generator : if s < CACHED_EXTENTS { 
                None 
            } else { 
                Some (Box::new (ExtentGenerator::new (Stage::from (s - 1))))
            },
            stage : stage,
            array_index : 0,
            insert_location : 0
        }
    }
}





const CACHED_EXTENTS : usize = 5;

static EXTENT_LENGTHS : [usize; CACHED_EXTENTS] = [0, 1, 2, 6, 24];

static EXTENTS : [&[u8]; CACHED_EXTENTS] = [
    &[],
    &[0],
    &[0, 1, 1, 0],
    &[2, 0, 1, 0, 2, 1, 0, 1, 2, 2, 1, 0, 1, 2, 0, 1, 0, 2],
    &[3, 2, 0, 1, 2, 3, 0, 1, 2, 0, 3, 1, 2, 0, 1, 3, 3, 0, 2, 1, 0, 3, 2, 1, 0, 2, 3, 1, 0, 2, 1, 3, 3, 0, 1, 2, 0, 3, 1, 2, 0, 1, 3, 2, 0, 1, 2, 3, 3, 2, 1, 0, 2, 3, 1, 0, 2, 1, 3, 0, 2, 1, 0, 3, 3, 1, 2, 0, 1, 3, 2, 0, 1, 2, 3, 0, 1, 2, 0, 3, 3, 1, 0, 2, 1, 3, 0, 2, 1, 0, 3, 2, 1, 0, 2, 3]
];








#[cfg(test)]
mod utils_tests {
    use crate::utils;
    use crate::{ Change, Stage };
    use crate::utils::ExtentIterator;

    use factorial::Factorial;

    #[test]
    fn closure () {
        assert_eq! (
            utils::closure (Change::from ("13425678")),
            vec! [
                Change::from ("12345678"),
                Change::from ("13425678"),
                Change::from ("14235678")
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("87654321")),
            vec! [
                Change::from ("12345678"),
                Change::from ("87654321")
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("1")),
            vec! [
                Change::from ("1"),
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("123456789")),
            vec! [
                Change::from ("123456789"),
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("4321675")),
            vec! [
                Change::from ("1234567"),
                Change::from ("4321675"),
                Change::from ("1234756"),
                Change::from ("4321567"),
                Change::from ("1234675"),
                Change::from ("4321756")
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("")),
            vec! [
                Change::from (""),
            ]
        );
    }

    #[test]
    fn extent () {
        for s in 1..9usize {
            let mut count = 0;

            for c in ExtentIterator::new (Stage::from (s)) {
                assert_eq! (c.stage (), Stage::from (s));

                count += 1;
            }

            assert_eq! (count, s.factorial ());
        }
    }
}
