use crate::{ Bell, Stage, Change, Touch, Transposition, MultiplicationIterator };
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

    fn music_score_without_alloc (&self, temp_change : &mut Change) -> usize {
        let mut score = 0;

        let mut iter = self.bell_iter ();

        while fill_from_iterator (&mut iter, temp_change.mut_slice ()) {
            score += temp_change.music_score ();
        }

        score
    }

    fn music_score (&self) -> usize {
        self.music_score_without_alloc (&mut Change::rounds (self.stage ()))
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
