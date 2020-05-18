use crate::{ Touch, Stage, Place, Bell, Transposition, Change, ChangeCollectIter, TouchIterator };
use crate::touch::RowIterator;

use factorial::Factorial;
use std::collections::HashMap;

use std::cmp::Ordering;

pub type ProofGroups = Vec<Vec<usize>>;

pub fn fill_from_iterator<T : Sized> (iter : &mut impl Iterator<Item = T>, slice : &mut [T]) -> bool {
    for i in 0..slice.len () {
        if let Some (v) = iter.next () {
            slice [i] = v;
        } else {
            return false;
        }
    }

    true
}

pub trait ProvingContext {
    fn prove_canonical<'a> (&mut self, iter : &impl TouchIterator<'a>, canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool;

    #[cfg_attr (tarpaulin, skip)]
    fn prove<'a> (&mut self, iter : &impl TouchIterator<'a>) -> bool {
        self.prove_canonical (iter, canon_copy)
    }

    fn prove_touch_canonical (&mut self, touch : &Touch, canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool{
        self.prove_canonical (&touch.iter (), canon)
    }

    fn prove_touch (&mut self, touch : &Touch) -> bool {
        self.prove_touch_canonical (touch, canon_copy)
    }
}






pub trait FullProvingContext : ProvingContext {
    fn full_prove_canonical<'a> (&mut self, iter : &impl TouchIterator<'a>, canon : impl FnMut(&[Bell], &mut Change) -> ()) -> ProofGroups;

    #[cfg_attr (tarpaulin, skip)]
    fn full_prove<'a> (&mut self, iter : &impl TouchIterator<'a>) -> ProofGroups {
        self.full_prove_canonical (iter, canon_copy)
    }

    fn full_prove_touch_canonical (&mut self, touch : &Touch, canon : impl FnMut(&[Bell], &mut Change) -> ()) -> ProofGroups {
        self.full_prove_canonical (&touch.iter (), canon)
    }

    fn full_prove_touch (&mut self, touch : &Touch) -> ProofGroups {
        self.full_prove_touch_canonical (touch, canon_copy)
    }
}






fn full_proof_from_iterator (iterator : impl Iterator<Item = (usize, usize)>) -> ProofGroups {
    let mut hash_map : HashMap<usize, Vec<usize>> = HashMap::with_capacity (20);

    for (root, value) in iterator {
        if !hash_map.contains_key (&root) {
            hash_map.insert (root, Vec::with_capacity (3));
            hash_map.get_mut (&root).unwrap ().push (root);
        }
        hash_map.get_mut (&root).unwrap ().push (value);
    }

    let mut vec = Vec::with_capacity (hash_map.len ());

    for (_, v) in hash_map.drain () {
        vec.push (v);
    }

    vec
}







#[derive(Eq, PartialEq, Debug)]
struct IndexedChange {
    pub index : usize,
    pub change : Change
}

impl Ord for IndexedChange {
    fn cmp (&self, other : &Self) -> Ordering {
        self.change.cmp (&other.change)
    }
}

impl PartialOrd for IndexedChange {
    fn partial_cmp (&self, other : &Self) -> Option<Ordering> {
        Some (self.cmp (other))
    }
}




pub struct NaiveProver { }

impl FullProvingContext for NaiveProver {
    fn full_prove_canonical<'a> (&mut self, iter : &impl TouchIterator<'a>, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> ProofGroups {
        let mut temporary_change = Change::rounds (iter.stage ());
        let mut temp_slice = vec![Bell::from (0); iter.stage ().as_usize ()];

        let mut indexed_changes : Vec<IndexedChange> = Vec::with_capacity (iter.length ());

        let mut bell_iter = iter.bell_iter ();
        let mut index = 0;

        while fill_from_iterator (&mut bell_iter, &mut temp_slice) {
            canon (&temp_slice, &mut temporary_change);

            indexed_changes.push (
                IndexedChange {
                    index : index,
                    change : temporary_change.clone ()
                }
            );

            index += 1;
        }

        indexed_changes.sort ();

        let mut truth = Vec::with_capacity (10);
        let mut temp_vec : Vec<usize> = Vec::with_capacity (5);
        let mut group_start_index = 0;

        temp_vec.push (0);
        for i in 1..indexed_changes.len () {
            if indexed_changes [i].change != indexed_changes [group_start_index].change {
                if temp_vec.len () > 1 {
                    truth.push (temp_vec.clone ());
                }

                group_start_index = i;
                temp_vec.clear ();
            }

            temp_vec.push (indexed_changes [i].index);
        }

        if temp_vec.len () > 1 {
            truth.push (temp_vec.clone ());
        }

        truth
    }
}

impl ProvingContext for NaiveProver {
    fn prove_canonical<'a> (&mut self, iter : &impl TouchIterator<'a>, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool {
        let mut temporary_change = Change::rounds (iter.stage ());

        let mut changes : Vec<Change> = Vec::with_capacity (iter.length ());

        let mut temp_slice = vec![Bell::from (0); iter.stage ().as_usize ()];
        let mut bell_iter = iter.bell_iter ();

        while fill_from_iterator (&mut bell_iter, &mut temp_slice) {
            canon (&temp_slice, &mut temporary_change);

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

    fn prove<'a> (&mut self, iter : &impl TouchIterator<'a>) -> bool {
        let mut changes : Vec<Change> = ChangeCollectIter::new (iter.bell_iter (), iter.stage ()).collect ();

        changes.sort ();

        for i in 1..changes.len () {
            if changes [i - 1] == changes [i] {
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
    fn prove_touch_canonical (&mut self, touch : &Touch, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool {
        assert_eq! (touch.stage, self.stage);

        let mut truth = true;
        let mut temporary_change = Change::rounds (touch.stage);

        for r in touch.row_iterator () {
            canon (&r.slice (), &mut temporary_change);

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

    fn prove_touch (&mut self, touch : &Touch) -> bool {
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

    fn prove_canonical<'a> (&mut self, iter : &impl TouchIterator<'a>, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool {
        assert_eq! (iter.stage (), self.stage);

        let mut truth = true;
        let mut temporary_change = Change::rounds (iter.stage ());

        let mut temp_slice = vec![Bell::from (0); iter.stage ().as_usize ()];
        let mut bell_iter = iter.bell_iter ();

        while fill_from_iterator (&mut bell_iter, &mut temp_slice) {
            canon (&temp_slice, &mut temporary_change);

            let hash = temporary_change.naive_hash ();

            if self.bit_map.get (hash) {
                truth = false;
                break;
            }

            self.bit_map.set_true (hash);
        }

        bell_iter = iter.bell_iter ();

        // Reset the hash map before returning
        while fill_from_iterator (&mut bell_iter, &mut temporary_change.mut_slice ()) {
            self.bit_map.set_false (temporary_change.naive_hash ());
        }

        truth
    }

    fn prove<'a> (&mut self, iter : &impl TouchIterator<'a>) -> bool {
        assert_eq! (iter.stage (), self.stage);

        let mut truth = true;

        let mut temporary_change = Change::rounds (iter.stage ());

        let mut bell_iter = iter.bell_iter ();

        while fill_from_iterator (&mut bell_iter, temporary_change.mut_slice ()) {
            let hash = temporary_change.naive_hash ();

            if self.bit_map.get (hash) {
                truth = false;
                break;
            }

            self.bit_map.set_true (hash);
        }

        bell_iter = iter.bell_iter ();

        // Reset the hash map before returning
        while fill_from_iterator (&mut bell_iter, temporary_change.mut_slice ()) {
            self.bit_map.set_false (temporary_change.naive_hash ());
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







type IndexType = i32;

pub struct CompactHashProver {
    stage : Stage,
    falseness_map : Vec<IndexType>,
    temporary_change : Change,
    temporary_slice : Vec<Bell>
}

impl CompactHashProver {
    pub fn from_stage (stage : Stage) -> CompactHashProver {
        CompactHashProver {
            stage : stage,
            falseness_map : vec![-1 as IndexType; stage.as_usize ().factorial ()],
            temporary_change : Change::rounds (stage),
            temporary_slice : vec![Bell::from (0); stage.as_usize ()]
        }
    }
}

impl FullProvingContext for CompactHashProver {
    fn full_prove_touch_canonical (&mut self, touch : &Touch, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> ProofGroups {
        let truth = full_proof_from_iterator (CompactHashTouchIterator {
            hash_prover : self,
            row_iterator : &mut touch.row_iterator (),
            canon_func : &mut canon
        }.map (|(x, y)| (x as usize, y as usize)));

        for r in touch.row_iterator () {
            canon (&r.slice (), &mut self.temporary_change);

            self.falseness_map [self.temporary_change.destructive_hash ()] = -1;
        }

        truth
    }

    fn full_prove_canonical<'a> (&mut self, iter : &impl TouchIterator<'a>, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> ProofGroups {
        let truth = full_proof_from_iterator (CompactHashIterator {
            hash_prover : self,
            bell_iter : &mut iter.bell_iter (),
            canon_func : &mut canon,
            index : 0
        }.map (|(x, y)| (x as usize, y as usize)));

        let mut bell_iter = iter.bell_iter ();

        while fill_from_iterator (&mut bell_iter, &mut self.temporary_slice) {
            canon (&self.temporary_slice, &mut self.temporary_change);

            self.falseness_map [self.temporary_change.destructive_hash ()] = -1;
        }

        truth
    }
}

impl ProvingContext for CompactHashProver {
    fn prove_touch_canonical (&mut self, touch : &Touch, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool {
        let truth = CompactHashTouchIterator {
            hash_prover : self,
            row_iterator : &mut touch.row_iterator (),
            canon_func : &mut canon
        }.next () == None;

        let mut c = Change::rounds (self.stage);

        for r in touch.row_iterator () {
            canon (&r.slice (), &mut c);

            self.falseness_map [c.destructive_hash ()] = -1;
        }

        truth
    }

    fn prove_canonical<'a> (&mut self, iter : &impl TouchIterator<'a>, mut canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool {
        let truth = CompactHashIterator {
            hash_prover : self,
            bell_iter : &mut iter.bell_iter (),
            canon_func : &mut canon,
            index : 0
        }.next () == None;

        let mut bell_iter = iter.bell_iter ();

        while fill_from_iterator (&mut bell_iter, &mut self.temporary_slice) {
            canon (&self.temporary_slice, &mut self.temporary_change);

            self.falseness_map [self.temporary_change.destructive_hash ()] = -1;
        }

        truth
    }
}

pub struct CompactHashIterator<'a, I : Iterator<Item = Bell>, T : FnMut(&[Bell], &mut Change) -> ()> {
    hash_prover : &'a mut CompactHashProver,
    bell_iter : &'a mut I,
    canon_func : &'a mut T,
    index : usize
}

impl<'a, I : Iterator<Item = Bell>, T : FnMut(&[Bell], &mut Change) -> ()> Iterator for CompactHashIterator<'_, I, T> {
    type Item = (IndexType, IndexType);

    fn next (&mut self) -> Option<Self::Item> {
        while fill_from_iterator (self.bell_iter, &mut self.hash_prover.temporary_slice) {
            (self.canon_func) (&self.hash_prover.temporary_slice, &mut self.hash_prover.temporary_change);

            let hash = self.hash_prover.temporary_change.destructive_hash ();
            let falseness_index = self.hash_prover.falseness_map [hash];

            if falseness_index == -1 {
                self.hash_prover.falseness_map [hash] = self.index as IndexType;

                self.index += 1;
            } else {
                let out = Some ((falseness_index, self.index as IndexType));

                self.index += 1;

                return out;
            }
        }

        None
    }
}

pub struct CompactHashTouchIterator<'a, T : FnMut(&[Bell], &mut Change) -> ()> {
    hash_prover : &'a mut CompactHashProver,
    row_iterator : &'a mut RowIterator<'a>,
    canon_func : &'a mut T
}

impl<T : FnMut(&[Bell], &mut Change) -> ()> Iterator for CompactHashTouchIterator<'_, T> {
    type Item = (IndexType, IndexType);

    fn next (&mut self) -> Option<Self::Item> {
        loop {
            match self.row_iterator.next () {
                Some (r) => {
                    (self.canon_func) (&r.slice (), &mut self.hash_prover.temporary_change);

                    let hash = self.hash_prover.temporary_change.destructive_hash ();
                    let falseness_index = self.hash_prover.falseness_map [hash];

                    if falseness_index == -1 {
                        self.hash_prover.falseness_map [hash] = r.index as IndexType;
                    } else {
                        return Some ((falseness_index, r.index as IndexType));
                    }
                }
                None => {
                    return None;
                }
            }
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

            map.set (i, false);
            assert_eq! (map.get (i), false);

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
pub fn canon_copy (slice : &[Bell], change : &mut Change) {
    change.overwrite_from_slice (slice);
}

pub fn canon_fixed_treble_cyclic (slice : &[Bell], change : &mut Change) {
    // We'll convert so that the first non-treble bell in the change is the 2
    let stage = slice.len ();

    if stage == 1 {
        change.set_bell (Place::from (0), Bell::from (0));

        return;
    }

    if slice [0] == Bell::from (0) {
        let shift = slice [1].as_isize () - 1;

        change.set_bell (Place::from (0), Bell::from (0));

        for i in 1..stage {
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

pub fn canon_full_cyclic (slice : &[Bell], change : &mut Change) {
    // We'll convert so that the first non-treble bell in the change is the 2
    let stage = slice.len ();

    // Nothing to be done if the stage is one
    if stage == 1 {
        change.set_bell (Place::from (0), Bell::from (0));

        return;
    }

    let shift = slice [0].as_usize ();

    for i in 0..stage {
        let new_bell = slice [i].as_usize () + stage - shift;

        change.set_bell (Place::from (i), Bell::from (new_bell % stage));
    }
}





#[cfg(test)]
mod tests {
    use crate::{ Touch };
    use crate::proving::*;

    fn full_proof_test_touches () -> Vec<(Touch, Vec<Vec<usize>>)> {
        vec![
            (Touch::from ("123"), vec![]),
            (Touch::from ("123456\n214365\n123456"), vec![]),
            (Touch::from ("123456\n214365\n123456\n123456"), vec![vec![0, 2]]),
        ]
    }

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
            assert_eq! (NaiveProver { }.prove_touch_canonical (&t, canon_copy), b);
            assert_eq! (NaiveProver { }.prove_touch (&t), b);
            assert_eq! (NaiveProver { }.prove (&mut t.iter ()), b);
        }
    }

    #[test]
    fn hash () {
        for (t, b) in test_touches () {
            if t.stage.as_usize () <= 8 {
                let mut prover = HashProver::from_stage (t.stage);

                assert_eq! (prover.prove_touch_canonical (&t, canon_copy), b);
                assert_eq! (prover.prove (&t.iter ()), b);
                assert_eq! (prover.prove_canonical (&t.iter (), canon_copy), b);
                assert_eq! (prover.prove_touch (&t), b);
            }
        }
    }

    #[test]
    fn compact_hash () {
        for (t, b) in test_touches () {
            assert_eq! (CompactHashProver::from_stage (t.stage).prove_touch (&t), b);
        }

        for (t, b) in full_proof_test_touches () {
            assert_eq! (CompactHashProver::from_stage (t.stage).full_prove_touch (&t), b);
        }
    }

    #[test]
    fn canon_func_fixed_treble_cyclic () {
        for (orig, canon) in &[
            ("1", "1"),
            ("132", "123"),
            ("123456", "123456"),
            ("42315678", "27813456"),
            ("71632548", "21854763"),
            ("18765432", "12876543"),
            ("15432876", "12876543"),
            ("87654321", "28765431"),
            ("4567890231", "2345678901")
        ] {
            let mut change = Change::new (vec! [Bell::from (20); canon.len ()]);

            canon_fixed_treble_cyclic (&Change::from (*orig).slice (), &mut change);

            assert_eq! (change, Change::from (*canon));
        }
    }

    #[test]
    fn canon_func_full_cyclic () {
        for (orig, canon) in &[
            ("1", "1"),
            ("132", "132"),
            ("123456", "123456"),
            ("42315678", "17862345"),
            ("71632548", "13854762"),
            ("87654321", "18765432"),
            ("4567890231", "1234567908")
        ] {
            let mut change = Change::new (vec! [Bell::from (20); canon.len ()]);

            canon_full_cyclic (&Change::from (*orig).slice (), &mut change);

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
                HashProver::from_stage (t.stage).prove_touch_canonical (&t, canon_fixed_treble_cyclic),
                *truth
            );
            assert_eq! (
                HashProver::from_stage (t.stage).prove_canonical (&t.iter (), canon_fixed_treble_cyclic),
                *truth
            );
            assert_eq! (
                CompactHashProver::from_stage (t.stage).prove_canonical (&t.iter (), canon_fixed_treble_cyclic),
                *truth
            );
            assert_eq! (
                CompactHashProver::from_stage (t.stage).prove_touch_canonical (&t, canon_fixed_treble_cyclic),
                *truth
            );
        }
    }

    #[test]
    fn full_canonical_proving () {
        for (touch, truth) in &[
            ("123\n132\n123", vec![vec![0, 1]]),
            ("12345678\n21436587\n17654328\n31547682\n12345678", vec![]),
            ("12345678\n21436587\n17654328\n31547628\n12345678", vec![vec![1, 3]]),
            (
                "12345678\n21436587\n17654328\n31547628\n13287654\n18765432\n12345678",
                vec![vec![1, 3], vec![2, 4, 5]]
            ),
        ] {
            let t = Touch::from (*touch);

            let mut compact_truth_touch = CompactHashProver::from_stage (t.stage)
                                .full_prove_touch_canonical (&t, canon_fixed_treble_cyclic);
            let mut compact_truth = CompactHashProver::from_stage (t.stage)
                                .full_prove_canonical (&t.iter (), canon_fixed_treble_cyclic);
            let mut naive_truth_touch = NaiveProver { }
                                .full_prove_touch_canonical (&t, canon_fixed_treble_cyclic);
            let mut naive_truth = NaiveProver { }
                                .full_prove_canonical (&t.iter (), canon_fixed_treble_cyclic);

            compact_truth_touch.sort ();
            naive_truth_touch.sort ();
            compact_truth.sort ();
            naive_truth.sort ();

            assert_eq! (compact_truth_touch, *truth);
            assert_eq! (naive_truth_touch, *truth);
            assert_eq! (compact_truth, *truth);
            assert_eq! (naive_truth, *truth);
        }
    }
}
