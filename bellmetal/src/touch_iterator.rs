use crate::{ Bell, Stage, Transposition, MultiplicationIterator };

pub trait TouchIterator<'a> {
    type BellIter : Iterator<Item = Bell>;
    type RuleoffIter : Iterator<Item = usize>;
    type CallIter : Iterator<Item = (usize, char)>;
    type MethodNameIter : Iterator<Item = (usize, &'a String)>;
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
