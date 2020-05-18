use crate::{ Change, Stage, Bell };
use std::iter::{ Fuse, Peekable };

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

pub fn extent (stage : Stage) -> impl Iterator<Item = Change> {
    ExtentIterator::new (Stage::from (stage))
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

            self.generator.fill (&mut bell_vec);

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

        match &mut self.recursive_generator {
            Some (g) => {
                self.insert_location += 1;

                if self.insert_location == stage {
                    self.insert_location = 0;

                    g.step ()
                } else {
                    true
                }
            }
            None => {
                self.array_index += 1;

                self.array_index < EXTENT_LENGTHS [stage]
            }
        }
    }

    pub fn fill (&self, slice : &mut [Bell]) {
        let stage = self.stage.as_usize ();

        match &self.recursive_generator {
            Some (g) => {
                // Generate the permutation recursively
                g.fill (slice);

                let mut temp = Bell::from (stage - 1);

                for i in self.insert_location..stage {
                    let t2 = slice [i];
                    slice [i] = temp;
                    temp = t2;
                }
            }
            None => {
                // Load the extent from static memory
                let extent_slice = EXTENTS [stage];

                for i in 0..stage {
                    slice [i] = Bell::from (extent_slice [self.array_index * stage + i] as usize);
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







pub struct AndNext<T : Iterator> {
    iter : Peekable<Fuse<T>>
}

impl<T : Iterator> AndNext<T> where T::Item : Copy {
    pub fn new (iter : T) -> AndNext<T> {
        AndNext {
            iter : iter.fuse ().peekable ()
        }
    }
}

impl<T : Iterator> Iterator for AndNext<T> where T::Item : Copy {
    type Item = (T::Item, Option<T::Item>);

    fn next (&mut self) -> Option<Self::Item> {
        match self.iter.next () {
            None => None,
            Some (v) => match self.iter.peek () {
                None => Some ((v, None)),
                Some (n) => Some ((v, Some (*n)))
            }
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn size_hint (&self) -> (usize, Option<usize>) {
        self.iter.size_hint ()
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
mod tests {
    use crate::{ Change, Stage, extent, closure };
    use crate::utils::{ AndNext };

    use factorial::Factorial;

    #[test]
    fn and_next_iter () {
        let mut iter = AndNext::new ([1, 2, 3].iter ());

        assert_eq! (iter.next (), Some ((&1, Some (&2))));
        assert_eq! (iter.next (), Some ((&2, Some (&3))));
        assert_eq! (iter.next (), Some ((&3, None)));
        assert_eq! (iter.next (), None);
    }

    #[test]
    fn change_closure () {
        assert_eq! (
            closure (Change::from ("13425678")),
            vec! [
                Change::from ("12345678"),
                Change::from ("13425678"),
                Change::from ("14235678")
            ]
        );

        assert_eq! (
            closure (Change::from ("87654321")),
            vec! [
                Change::from ("12345678"),
                Change::from ("87654321")
            ]
        );

        assert_eq! (
            closure (Change::from ("1")),
            vec! [
                Change::from ("1"),
            ]
        );

        assert_eq! (
            closure (Change::from ("123456789")),
            vec! [
                Change::from ("123456789"),
            ]
        );

        assert_eq! (
            closure (Change::from ("4321675")),
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
            closure (Change::from ("")),
            vec! [
                Change::from (""),
            ]
        );
    }

    #[test]
    fn extent_gen () {
        for s in 1..9usize {
            let mut count = 0;

            for c in extent (Stage::from (s)) {
                assert_eq! (c.stage (), Stage::from (s));

                count += 1;
            }

            assert_eq! (count, s.factorial ());
        }
    }
}
