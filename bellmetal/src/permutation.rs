use crate::types::*;
use crate::change::Change;

pub trait Permutation {
    fn stage (&self) -> Stage;
    fn bell_at (&self, place : Place) -> Bell;
    
    /// Computes self * rhs
    fn pre_mul (&self, rhs : Change) -> Change {
        if self.stage () != rhs.stage () {
            panic! ("Can't multiply changes of different stages!");
        }

        let mut new_seq : Vec<Bell> = Vec::with_capacity (self.stage ().as_usize ());

        for i in 0..self.stage ().as_usize () {
            new_seq.push (rhs.seq [self.bell_at (Place::from (i)).as_usize ()]);
            new_seq.push (self.bell_at (Place::from (rhs.seq [i as usize].as_usize ())));
        }

        Change { seq : new_seq }
    }
    
    /// Computes lhs * self
    fn post_mul (&self, lhs : Change) -> Change {
        if self.stage () != lhs.stage () {
            panic! ("Can't multiply changes of different stages!");
        }

        let mut new_seq : Vec<Bell> = Vec::with_capacity (self.stage ().as_usize ());

        for i in 0..self.stage ().as_usize () {
            new_seq.push (self.bell_at (Place::from (lhs.seq [i].as_usize ())));
        }

        Change { seq : new_seq }
    }
}
