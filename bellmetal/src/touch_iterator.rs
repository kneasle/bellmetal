use crate::{ Bell, Stage, Change, Touch, Transposition, MultiplicationIterator, MusicScoring };
use crate::proving::fill_from_iterator;

use core::iter::Chain;

pub trait TouchIterator<'a> {
    type BellIter : Iterator<Item = Bell>;
    type RuleoffIter : Iterator<Item = usize>;
    type CallIter : Iterator<Item = (usize, char)>;
    type MethodNameIter : Iterator<Item = (usize, &'a str)>;
    type LeftoverChangeIter : Iterator<Item = Bell>;

    fn bell_iter (&self) -> Self::BellIter;
    fn ruleoff_iter (&self) -> Self::RuleoffIter;
    fn call_iter (&self) -> Self::CallIter;
    fn method_name_iter (&self) -> Self::MethodNameIter;

    fn stage (&self) -> Stage;
    fn leftover_change_iter (&self) -> Self::LeftoverChangeIter;

    fn length (&self) -> usize;

    fn transfigure<T> (self, transposition : &'a T) -> TransfiguredTouchIterator<Self, T>
            where Self : Sized, T : Transposition {
        TransfiguredTouchIterator::new (self, transposition)
    }

    fn chain<I : TouchIterator<'a>> (&'a self, other : &'a I) -> ChainedTouchIterator<Self, I> where Self : Sized {
        ChainedTouchIterator::new (self, other)
    }

    fn collect (self) -> Touch where Self : Sized {
        Touch::from_iterator (&self)
    }

    fn music_score_without_alloc<T : MusicScoring> (&self, temp_change : &mut Change) -> usize {
        let mut score = 0;

        let mut iter = self.bell_iter ();

        while fill_from_iterator (&mut iter, temp_change.mut_slice ()) {
            score += temp_change.music_score::<T> ();
        }

        score
    }

    fn music_score<T : MusicScoring> (&self) -> usize {
        self.music_score_without_alloc::<T> (&mut Change::rounds (self.stage ()))
    }
}






pub struct CallShift<I : Iterator<Item = (usize, char)>> {
    iter : I,
    shift : usize
}

impl<T : Iterator<Item = (usize, char)>> Iterator for CallShift<T> {
    type Item = (usize, char);

    fn next (&mut self) -> Option<(usize, char)> {
        match self.iter.next () {
            Some ((i, c)) => Some ((i + self.shift, c)),
            None => None
        }
    }

    fn size_hint (&self) -> (usize, Option<usize>) {
        self.iter.size_hint ()
    }
}



pub struct RuleoffShift<I : Iterator<Item = usize>> {
    iter : I,
    shift : usize
}

impl<T : Iterator<Item = usize>> Iterator for RuleoffShift<T> {
    type Item = usize;

    fn next (&mut self) -> Option<usize> {
        match self.iter.next () {
            Some (v) => Some (v + self.shift),
            None => None
        }
    }

    fn size_hint (&self) -> (usize, Option<usize>) {
        self.iter.size_hint ()
    }
}



pub struct MethodNameShift<'a, I : Iterator<Item = (usize, &'a str)>> {
    iter : I,
    shift : usize
}

impl<'a, T : Iterator<Item = (usize, &'a str)>> Iterator for MethodNameShift<'a, T> {
    type Item = (usize, &'a str);

    fn next (&mut self) -> Option<(usize, &'a str)> {
        match self.iter.next () {
            Some ((i, s)) => Some ((i + self.shift, s)),
            None => None
        }
    }

    fn size_hint (&self) -> (usize, Option<usize>) {
        self.iter.size_hint ()
    }
}



pub struct ChainedTouchIterator<'a, I : TouchIterator<'a>, J : TouchIterator<'a>> {
    first : &'a I,
    second : &'a J,
    first_len : usize
}

impl<'a, I : TouchIterator<'a>, J : TouchIterator<'a>> ChainedTouchIterator<'a, I, J> {
    pub fn new (first : &'a I, second : &'a J) -> ChainedTouchIterator<'a, I, J> {
        ChainedTouchIterator {
            first_len : first.length (),
            first : first,
            second : second
        }
    }
}

impl<'a, I : TouchIterator<'a>, J : TouchIterator<'a>> TouchIterator<'a> for ChainedTouchIterator<'a, I, J> {
    type BellIter = Chain<I::BellIter, J::BellIter>;
    type RuleoffIter = Chain<I::RuleoffIter, RuleoffShift<J::RuleoffIter>>;
    type CallIter = Chain<I::CallIter, CallShift<J::CallIter>>;
    type MethodNameIter = Chain<I::MethodNameIter, MethodNameShift<'a, J::MethodNameIter>>;
    type LeftoverChangeIter = J::LeftoverChangeIter;

    fn bell_iter (&self) -> Self::BellIter {
        self.first.bell_iter ().chain (self.second.bell_iter ())
    }

    fn ruleoff_iter (&self) -> Self::RuleoffIter {
        self.first.ruleoff_iter ().chain (RuleoffShift {
            iter : self.second.ruleoff_iter (),
            shift : self.first_len
        })
    }

    fn call_iter (&self) -> Self::CallIter {
        self.first.call_iter ().chain (CallShift {
            iter : self.second.call_iter (),
            shift : self.first_len
        })
    }

    fn method_name_iter (&self) -> Self::MethodNameIter {
        self.first.method_name_iter ().chain (MethodNameShift {
            iter : self.second.method_name_iter (),
            shift : self.first_len
        })
    }

    fn stage (&self) -> Stage {
        self.first.stage ()
    }

    fn length (&self) -> usize {
        self.first_len + self.second.length ()
    }

    fn leftover_change_iter (&self) -> Self::LeftoverChangeIter {
        self.second.leftover_change_iter ()
    }
}







pub struct MultiChainBellIterator<'a, T : TouchIterator<'a>> {
    iterators : &'a [T],
    iter_index : usize,
    current_iterator : T::BellIter
}

impl<'a, T : TouchIterator<'a>> MultiChainBellIterator<'a, T> {
    pub fn new (iterators : &'a [T]) -> MultiChainBellIterator<'a, T> {
        MultiChainBellIterator {
            iterators : iterators,
            iter_index : 0,
            current_iterator : iterators [0].bell_iter ()
        }
    }
}

impl<'a, T : TouchIterator<'a>> Iterator for MultiChainBellIterator<'a, T> {
    type Item = Bell;

    fn next (&mut self) -> Option<Bell> {
        let mut next_value = self.current_iterator.next ();

        while next_value == None {
            self.iter_index += 1;

            if self.iter_index >= self.iterators.len () {
                return None;
            }

            self.current_iterator = self.iterators [self.iter_index].bell_iter ();

            next_value = self.current_iterator.next ();
        }

        next_value
    }
}



pub struct MultiChainRuleoffIterator<'a, T : TouchIterator<'a>> {
    iterators : &'a [T],
    iter_index : usize,
    current_iterator : T::RuleoffIter,
    length_used : usize
}

impl<'a, T : TouchIterator<'a>> MultiChainRuleoffIterator<'a, T> {
    pub fn new (iterators : &'a [T]) -> MultiChainRuleoffIterator<'a, T> {
        MultiChainRuleoffIterator {
            iterators : iterators,
            iter_index : 0,
            current_iterator : iterators [0].ruleoff_iter (),
            length_used : 0
        }
    }
}

impl<'a, T : TouchIterator<'a>> Iterator for MultiChainRuleoffIterator<'a, T> {
    type Item = usize;

    fn next (&mut self) -> Option<usize> {
        let mut next_value = self.current_iterator.next ();

        while next_value == None {
            self.length_used += self.iterators [self.iter_index].length ();

            self.iter_index += 1;

            if self.iter_index >= self.iterators.len () {
                return None;
            }

            self.current_iterator = self.iterators [self.iter_index].ruleoff_iter ();

            next_value = self.current_iterator.next ();
        }

        if let Some (i) = next_value {
            Some (i + self.length_used)
        } else {
            panic! ("This code shouldn't be executed");
        }
    }
}



pub struct MultiChainCallIterator<'a, T : TouchIterator<'a>> {
    iterators : &'a [T],
    iter_index : usize,
    current_iterator : T::CallIter,
    length_used : usize
}

impl<'a, T : TouchIterator<'a>> MultiChainCallIterator<'a, T> {
    pub fn new (iterators : &'a [T]) -> MultiChainCallIterator<'a, T> {
        MultiChainCallIterator {
            iterators : iterators,
            iter_index : 0,
            current_iterator : iterators [0].call_iter (),
            length_used : 0
        }
    }
}

impl<'a, T : TouchIterator<'a>> Iterator for MultiChainCallIterator<'a, T> {
    type Item = (usize, char);

    fn next (&mut self) -> Option<(usize, char)> {
        let mut next_value = self.current_iterator.next ();

        while next_value == None {
            self.length_used += self.iterators [self.iter_index].length ();

            self.iter_index += 1;

            if self.iter_index >= self.iterators.len () {
                return None;
            }

            self.current_iterator = self.iterators [self.iter_index].call_iter ();

            next_value = self.current_iterator.next ();
        }

        if let Some ((i, c)) = next_value {
            Some ((i + self.length_used, c))
        } else {
            panic! ("This code shouldn't be executed");
        }
    }
}



pub struct MultiChainMethodNameIterator<'a, T : TouchIterator<'a>> {
    iterators : &'a [T],
    iter_index : usize,
    current_iterator : T::MethodNameIter,
    length_used : usize
}

impl<'a, T : TouchIterator<'a>> MultiChainMethodNameIterator<'a, T> {
    pub fn new (iterators : &'a [T]) -> MultiChainMethodNameIterator<'a, T> {
        MultiChainMethodNameIterator {
            iterators : iterators,
            iter_index : 0,
            current_iterator : iterators [0].method_name_iter (),
            length_used : 0
        }
    }
}

impl<'a, T : TouchIterator<'a>> Iterator for MultiChainMethodNameIterator<'a, T> {
    type Item = (usize, &'a str);

    fn next (&mut self) -> Option<(usize, &'a str)> {
        let mut next_value = self.current_iterator.next ();

        while next_value == None {
            self.length_used += self.iterators [self.iter_index].length ();

            self.iter_index += 1;

            if self.iter_index >= self.iterators.len () {
                return None;
            }

            self.current_iterator = self.iterators [self.iter_index].method_name_iter ();

            next_value = self.current_iterator.next ();
        }

        if let Some ((i, c)) = next_value {
            Some ((i + self.length_used, c))
        } else {
            panic! ("This code shouldn't be executed");
        }
    }
}



pub struct MultiChainTouchIterator<'a, T : TouchIterator<'a>> {
    iterators : &'a [T]
}

impl<'a, T : TouchIterator<'a>> MultiChainTouchIterator<'a, T> {
    pub fn new (iterators : &'a [T]) -> MultiChainTouchIterator<'a, T> {
        assert! (iterators.len () > 0);

        let stage = iterators [0].stage ();
        for i in 1..iterators.len () {
            assert_eq! (iterators [i].stage (), stage);
        }

        MultiChainTouchIterator {
            iterators : iterators
        }
    }
}

impl<'a, T : TouchIterator<'a>> TouchIterator<'a> for MultiChainTouchIterator<'a, T> {
    type BellIter = MultiChainBellIterator<'a, T>;
    type RuleoffIter = MultiChainRuleoffIterator<'a, T>;
    type CallIter = MultiChainCallIterator<'a, T>;
    type MethodNameIter = MultiChainMethodNameIterator<'a, T>;
    type LeftoverChangeIter = T::LeftoverChangeIter;

    fn bell_iter (&self) -> Self::BellIter {
        MultiChainBellIterator::new (self.iterators)
    }

    fn ruleoff_iter (&self) -> Self::RuleoffIter {
        MultiChainRuleoffIterator::new (self.iterators)
    }

    fn call_iter (&self) -> Self::CallIter {
        MultiChainCallIterator::new (self.iterators)
    }

    fn method_name_iter (&self) -> Self::MethodNameIter {
        MultiChainMethodNameIterator::new (self.iterators)
    }

    fn leftover_change_iter (&self) -> Self::LeftoverChangeIter {
        self.iterators [self.iterators.len () - 1].leftover_change_iter ()
    }

    fn stage (&self) -> Stage {
        self.iterators [0].stage ()
    }

    fn length (&self) -> usize {
        self.iterators.iter ().map (|x| x.length ()).sum ()
    }
}






pub struct TransfiguredTouchIterator<'a, I : TouchIterator<'a>, T : Transposition> {
    iterator : I,
    transposition : &'a T
}

impl<'a, I : TouchIterator<'a>, T : Transposition> TransfiguredTouchIterator<'a, I, T> {
    fn new (iterator : I, transposition : &'a T) -> TransfiguredTouchIterator<'a, I, T> {
        TransfiguredTouchIterator {
            iterator : iterator,
            transposition : transposition
        }
    }
}

impl<'a, I : TouchIterator<'a>, T : Transposition> TouchIterator<'a> for TransfiguredTouchIterator<'a, I, T> {
    type BellIter = MultiplicationIterator<'a, I::BellIter>;
    type RuleoffIter = I::RuleoffIter;
    type CallIter = I::CallIter;
    type MethodNameIter = I::MethodNameIter;
    type LeftoverChangeIter = MultiplicationIterator<'a, I::LeftoverChangeIter>;

    fn bell_iter (&self) -> Self::BellIter {
        MultiplicationIterator::new (&self.transposition.slice (), self.iterator.bell_iter ())
    }

    fn ruleoff_iter (&self) -> Self::RuleoffIter {
        self.iterator.ruleoff_iter ()
    }

    fn call_iter (&self) -> Self::CallIter {
        self.iterator.call_iter ()
    }

    fn method_name_iter (&self) -> Self::MethodNameIter {
        self.iterator.method_name_iter ()
    }

    fn stage (&self) -> Stage {
        self.iterator.stage ()
    }

    fn length (&self) -> usize {
        self.iterator.length ()
    }

    fn leftover_change_iter (&self) -> Self::LeftoverChangeIter {
        MultiplicationIterator::new (&self.transposition.slice (), self.iterator.leftover_change_iter ())
    }
}
