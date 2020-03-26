use crate::{ Bell, Place, Stage, Stroke, Change, Transposition };

use std::fmt;
use std::ops::Index;
use std::cmp;
use std::cmp::Ordering;






#[derive(PartialEq, Eq)]
pub struct RunSection {
    start : isize,
    centre : isize,
    end : isize,
    bell_start : Bell,
    bell_end : Bell,
    stroke : Stroke
}

impl Ord for RunSection {
    fn cmp (&self, other : &Self) -> Ordering {
        self.bell_end.cmp (&other.bell_end)
    }
}

impl PartialOrd for RunSection {
    fn partial_cmp (&self, other : &Self) -> Option<Ordering> {
        Some (self.cmp (other))
    }
}





#[derive(PartialEq, Eq)]
pub struct CoursingOrder {
    order : Vec<Bell> // order will always start with the heaviest bell in the coursing order
}

impl CoursingOrder {
    pub fn overwrite_from_string (&mut self, string : &str) {
        self.order.clear ();
        self.order.reserve (string.len ());

        for c in string.chars () {
            self.order.push (Bell::from (c));
        }
    }

    pub fn overwrite_from_iterator (&mut self, iterator : &mut impl CoursingOrderIterator) {
        let heaviest_bell = {
            let mut h = 0;

            // Find what the heaviest bell is
            for _ in 0..iterator.length () {
                let b = iterator.next ().as_usize ();

                if b > h {
                    h = b;
                }
            }

            Bell::from (h)
        };

        // Seek to the heaviest bell so the iterator is rotated with a horrendous loop with
        // side effects in the guard
        while iterator.next () != heaviest_bell { }

        // Copy the iterator into the vector
        let len = iterator.length ();

        self.order.clear ();
        self.order.reserve (len);

        self.order.push (heaviest_bell);
        for _ in 0..len - 1 {
            let x = iterator.next ();
            
            self.order.push (x);
        }
    }

    pub fn overwrite_from_leadhead (&mut self, lh : &Change) {
        self.overwrite_from_iterator (&mut LeadheadCoursingOrderIterator::new (lh));
    }

    fn test_run_segment_up (&self, root : isize, side : isize, vec : &mut Vec<RunSection>) {
        let mut len = 0;
        let mut current_index = 0;
        let mut last_index = 0;

        let a = self [root].as_isize ();

        for (l, index) in ZigZagIterator::new (root, side).enumerate () {
            if self [index].as_isize () != a + l as isize {
                len = l;

                break;
            }

            last_index = current_index;
            current_index = index;
        }

        if len >= 4 {
            vec.push (
                RunSection {
                    start : cmp::min (current_index, last_index),
                    centre : root,
                    end : cmp::max (current_index, last_index),
                    bell_start : self [current_index],
                    bell_end : self [root],
                    stroke : if root < side { Stroke::Back } else { Stroke::Hand }
                }
            );
        }
    }

    fn test_run_segment_down (&self, root : isize, side : isize, vec : &mut Vec<RunSection>) {
        let mut len = 0;
        let mut current_index = 0;
        let mut last_index = 0;

        let a = self [root].as_isize ();

        for (l, index) in ZigZagIterator::new (root, side).enumerate () {
            if self [index].as_isize () != a - l as isize {
                len = l;

                break;
            }

            last_index = current_index;
            current_index = index;
        }

        if len >= 4 {
            vec.push (
                RunSection {
                    start : cmp::min (current_index, last_index),
                    centre : root,
                    end : cmp::max (current_index, last_index),
                    bell_start : self [current_index],
                    bell_end : self [root],
                    stroke : if root < side { Stroke::Back } else { Stroke::Hand }
                }
            );
        }
    }

    fn get_run_sections (&self) -> Vec<RunSection> {
        let mut run_sections : Vec<RunSection> = Vec::with_capacity (10);

        for i in 0..self.order.len () as isize {
            let a = self [i - 1].as_isize ();
            let b = self [i].as_isize ();

            if b - a == 1 {
                self.test_run_segment_up (i - 1, i, &mut run_sections);
                self.test_run_segment_down (i, i - 1, &mut run_sections);
            }
            if a - b == 1 {
                self.test_run_segment_up (i, i - 1, &mut run_sections);
                self.test_run_segment_down (i - 1, i, &mut run_sections);
            }
        }

        run_sections.sort ();

        run_sections
    }

    pub fn into_string (&self, string : &mut String) {
        string.reserve (self.order.len ());

        for b in &self.order {
            string.push (b.as_char ());
        }
    }

    pub fn to_string (&self) -> String {
        let mut s = String::with_capacity (0);

        self.into_string (&mut s);

        s
    }

    pub fn canonical_string (&self) -> String {
        let mut string = String::with_capacity (100);

        string.push_str ("CO: <");

        for b in self.order.iter ().skip (1) {
            string.push (b.as_char ());
        }

        string.push_str (">");

        for r in self.get_run_sections () {
            let a = r.bell_start.as_usize ();
            let b = r.bell_end.as_usize ();

            string.push_str (" ");

            if b > a {
                for i in a..=b {
                    string.push (Bell::from (i).as_char ());
                }
            } else {
                for i in (b..=a).rev () {
                    string.push (Bell::from (i).as_char ());
                }
            }

            if r.stroke == Stroke::Hand {
                string.push ('h');
            }
        }

        string
    }
}

impl CoursingOrder {
    pub fn empty () -> CoursingOrder {
        CoursingOrder {
            order : Vec::with_capacity (0)
        }
    }
    
    pub fn from_leadhead (lh : &Change) -> CoursingOrder {
        let mut co = CoursingOrder::empty ();

        co.overwrite_from_leadhead (lh);

        co
    }

    pub fn from_iterator<T> (iterator : &mut T) -> CoursingOrder where T : CoursingOrderIterator {
        let mut co = CoursingOrder::empty ();

        co.overwrite_from_iterator (iterator);

        co
    }
}

impl Index<isize> for CoursingOrder {
    type Output = Bell;

    fn index (&self, x : isize) -> &Bell {
        let l = self.order.len () as isize;

        &self.order [(((x % l) + l) % l) as usize]
    }
}

impl From<&str> for CoursingOrder {
    fn from (s : &str) -> CoursingOrder {
        let mut coursing_order = CoursingOrder::empty ();

        coursing_order.overwrite_from_string (s);

        coursing_order
    }
}

impl fmt::Debug for CoursingOrder {
    fn fmt (&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity (self.order.len () + 2);

        for b in &self.order {
            s.push (b.as_char ());
        }

        write! (f, "<{}>", s)
    }
}











struct ZigZagIterator {
    current_value : isize,
    next_value : isize
}

impl ZigZagIterator {
    pub fn new (current_value : isize, next_value : isize) -> ZigZagIterator {
        ZigZagIterator {
            current_value : current_value,
            next_value : next_value
        }
    }
}

impl Iterator for ZigZagIterator {
    type Item = isize;

    fn next (&mut self) -> Option<isize> {
        let current_value = self.current_value;

        self.current_value = self.next_value;
        if self.next_value > current_value {
            self.next_value = current_value - 1;
        } else {
            self.next_value = current_value + 1;
        }

        Some (current_value)
    }
}






pub trait CoursingOrderIterator {
    fn next (&mut self) -> Bell;
    fn length (&self) -> usize;

    fn collect (&mut self) -> CoursingOrder where Self : std::marker::Sized {
        CoursingOrder::from_iterator (self)
    }
}







pub struct BasicCoursingOrderIterator<'a> {
    coursing_order : &'a CoursingOrder,
    index : usize
}

impl BasicCoursingOrderIterator<'_> {
    pub fn new<'a> (coursing_order : &'a CoursingOrder) -> BasicCoursingOrderIterator<'a> {
        BasicCoursingOrderIterator {
            coursing_order : coursing_order,
            index : 0
        }
    }
}

impl<'a> CoursingOrderIterator for BasicCoursingOrderIterator<'a> {
    fn next (&mut self) -> Bell {
        let b = self.coursing_order [self.index as isize];

        self.index += 1;

        b
    }

    fn length (&self) -> usize {
        self.coursing_order.order.len ()
    }
}







pub struct LeadheadCoursingOrderIterator<'a> {
    leadhead : &'a Change,
    iterator : PlainCoursingOrderIterator
}

impl LeadheadCoursingOrderIterator<'_> {
    pub fn new<'a> (leadhead : &'a Change) -> LeadheadCoursingOrderIterator<'a> {
        LeadheadCoursingOrderIterator {
            leadhead : leadhead,
            iterator : PlainCoursingOrderIterator::new (leadhead.stage ())
        }
    }
}

impl<'a> CoursingOrderIterator for LeadheadCoursingOrderIterator<'a> {
    fn next (&mut self) -> Bell {
        self.leadhead.bell_at (Place::from (self.iterator.next ().as_usize ()))
    }

    fn length (&self) -> usize {
        self.iterator.length ()
    }
}








pub struct PlainCoursingOrderIterator {
    stage : Stage,
    current_bell : usize,
    is_going_down : bool
}

impl PlainCoursingOrderIterator {
    pub fn new (stage : Stage) -> PlainCoursingOrderIterator {
        PlainCoursingOrderIterator {
            stage : stage,
            current_bell : (stage.as_usize () + 1 & !1) - 2,
            is_going_down : true
        }
    }
}

impl CoursingOrderIterator for PlainCoursingOrderIterator {
    fn next (&mut self) -> Bell {
        if self.stage == Stage::TWO {
            return Bell::from (1);
        }

        let current_bell = self.current_bell;

        if self.is_going_down {
            self.current_bell -= 2;

            if self.current_bell == 0 {
                self.is_going_down = false;
                self.current_bell = 1;
            }
        } else {
            self.current_bell += 2;
            
            if self.current_bell >= self.stage.as_usize () {
                self.is_going_down = true;
                self.current_bell = (self.stage.as_usize () + 1 & !1) - 2;
            }
        }

        Bell::from (current_bell)
    }

    fn length (&self) -> usize {
        self.stage.as_usize () - 1
    }
}








#[cfg(test)]
mod co_tests {
    use crate::{ 
        Stage, Change,
        CoursingOrder, 
        CoursingOrderIterator,
        BasicCoursingOrderIterator, PlainCoursingOrderIterator, 
        LeadheadCoursingOrderIterator
    };

    use crate::coursing_order::ZigZagIterator;

    #[test]
    fn zig_zag_iterator () {
        for s in -20..20 {
            // Test upwards
            let mut iter = ZigZagIterator::new (s, s + 1);

            assert_eq! (iter.next (), Some (s));

            for i in 1..100 {
                assert_eq! (iter.next (), Some (s + i));
                assert_eq! (iter.next (), Some (s - i));
            }

            // Test downwards
            let mut iter = ZigZagIterator::new (s, s - 1);

            assert_eq! (iter.next (), Some (s));

            for i in 1..100 {
                assert_eq! (iter.next (), Some (s - i));
                assert_eq! (iter.next (), Some (s + i));
            }
        }
    }

    #[test]
    fn plain_iterator () {
        for order in &[
            "324",
            "5324",
            "53246",
            "753246",
            "7532468",
            "97532468",
            "975324680"
        ] {
            let stage = Stage::from (order.chars ().count () + 1);

            let mut a = PlainCoursingOrderIterator::new (stage);
            let mut b = order.chars ().cycle ();

            for _ in 0..10 {
                let l = a.next ().as_char ();
                let r = b.next ().unwrap ();

                assert_eq! (l, r);
            }

            assert_eq! (a.length (), stage.as_usize () - 1);
        }
    }

    #[test]
    fn basic_iterator () {
        for order in &[
            "8753462",
            "98762453",
            "5324",
            "65342",
            "8657234",
            "2"
        ] {
            let co = CoursingOrder::from (*order);

            assert_eq! (BasicCoursingOrderIterator::new (&co).collect (), co);
        }
    }

    #[test]
    fn leadhead_iterator () {
        for (lh, order) in &[
            ("12", "2"),
            ("12345", "5324"),
            ("15432", "5324"),
            ("4567123", "723165"),
            ("12348765", "8324756"),
            ("1209876543", "029753468"),
        ] {
            assert_eq! (
                LeadheadCoursingOrderIterator::new (&Change::from (*lh)).collect (),
                CoursingOrder::from (*order)
            );
        }
    }

    #[test]
    fn canonical_strings () {
        for (order, canon) in &[
            ("087953246", "CO: <87953246> 65432h 0987h"),
            ("097246538", "CO: <97246538> 23456 7890"),
            ("TE976824530", "CO: <E976824530> 2345h 9876h 90ET"),
            ("029753468", "CO: <29753468> 09876543"),
            ("8753462", "CO: <753462> 76543")
        ] {
            assert_eq! (CoursingOrder::from (*order).canonical_string (), *canon);
        }
    }
}
