use crate::types::{ Stage, Bell };
use crate::place_notation::PlaceNotation;
use crate::change::{ Change, ChangeAccumulator };
use crate::transposition::Transposition;

pub struct Row<'a> {
    index : usize,
    is_ruled_off : bool,
    bells : &'a [Bell]
}

impl Transposition for Row<'_> {
    fn slice (&self) -> &[Bell] {
        self.bells
    }
}






pub struct Touch {
    pub stage : Stage,
    pub length : usize,

    // This will have length 1 more than Touch.length, since it also stores the 'left-over' change
    // that would be the first change after the touch finishes
    bells : Vec<Bell>,
    ruleoffs : Vec<usize>
}

impl Touch {
    fn row_iterator<'a> (&'a self) -> RowIterator<'a> {
        RowIterator::new (self)
    }

    fn row_at (&self, index : usize) -> Row {
        let stage = self.stage.as_usize ();

        Row {
            index : index,
            is_ruled_off : match self.ruleoffs.binary_search (&index) {
                Ok (_) => { true }
                Err (_) => { false }
            },
            bells : &self.bells [index * stage .. (index + 1) * stage]
        }
    }

    fn leftover_row (&self) -> Row {
        self.row_at (self.length)
    }

    fn to_string (&self) -> String {
        let stage = self.stage.as_usize ();

        let mut s = String::with_capacity (stage * self.length + self.length - 1);

        for i in 0..self.length {
            for j in 0..stage {
                s.push (self.bells [i * stage + j].as_char ());
            }
            
            if i != self.length - 1 {
                s.push ('\n');
            }
        }

        s
    }
}

impl From<&[PlaceNotation]> for Touch {
    fn from (place_notations : &[PlaceNotation]) -> Touch {
        let length = place_notations.len () + 1;

        if length == 1 {
            panic! ("Touch must be made of at least one place notation");
        }

        let stage = {
            let mut stage = None;

            for p in place_notations {
                match stage {
                    Some (s) => {
                        if s != p.stage {
                            panic! ("Every place notation of a touch must be of the same stage");
                        }
                    }
                    None => { stage = Some (p.stage) }
                }
            }

            stage.unwrap ().as_usize ()
        };

        let bells = {
            let mut bells : Vec<Bell> = Vec::with_capacity (length * stage);
            
            let mut accumulator : ChangeAccumulator = ChangeAccumulator::new (Stage::from (stage));

            macro_rules! add_change {
                () => {
                    for b in accumulator.total ().iterator () {
                        bells.push (b);
                    }
                }
            }
            
            add_change! ();
            for p in place_notations {
                accumulator.accumulate_iterator (p.iterator ());
                add_change! ();
            }

            bells
        };
        
        Touch {
            stage : Stage::from (stage),
            length : length - 1,

            bells : bells,
            ruleoffs : Vec::with_capacity (0)
        }
    }
}

impl From<&str> for Touch {
    fn from (string : &str) -> Touch {
        let (stage, length) = {
            let mut length = 0;
            let mut potential_stage = None;

            for line in string.lines () {
                match potential_stage {
                    Some (s) => {
                        if s != line.len () {
                            panic! ("Every row of a stage must be the same length");
                        }
                    }
                    None => { potential_stage = Some (line.len ()); }
                }

                length += 1;
            }

            match potential_stage {
                Some (s) => { (s, length) }
                None => { panic! ("Cannot create an empty touch"); }
            }
        };
        
        let bells = {
            let mut bells : Vec<Bell> = Vec::with_capacity (length * stage);
            
            for line in string.lines () {
                for c in line.chars () {
                    bells.push (Bell::from (c));
                }
            }

            bells
        };
        
        Touch {
            stage : Stage::from (stage),
            length : length,

            bells : bells,
            ruleoffs : Vec::with_capacity (0)
        }
    }
}






struct RowIterator<'a> {
    touch : &'a Touch,
    row_index : usize,
    ruleoff_index : usize
}

impl RowIterator<'_> {
    fn new (touch : &Touch) -> RowIterator {
        RowIterator {
            touch : touch,
            row_index : 0,
            ruleoff_index : 0
        }
    }
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = Row<'a>;

    fn next (&mut self) -> Option<Row<'a>> {
        let stage = self.touch.stage.as_usize ();
        let index = self.row_index;

        if index < self.touch.length {
            let is_ruleoff = if self.ruleoff_index >= self.touch.ruleoffs.len () { false } 
                            else { self.touch.ruleoffs [self.ruleoff_index] == index };

            let row = Row {
                index : index,
                is_ruled_off : is_ruleoff,
                bells : &self.touch.bells [index * stage .. (index + 1) * stage]
            };

            self.row_index += 1;
            if is_ruleoff {
                self.ruleoff_index += 1;
            }

            Some (row)
        } else {
            None
        }
    }
}






struct TransfiguredTouchIterator<'a> {
    transposition : &'a Change,
    touch : &'a Touch,

    next_bell_index : usize,
    next_ruleoff_index : usize
}

impl TransfiguredTouchIterator<'_> {
    pub fn new<'a> (change : &'a Change, touch : &'a Touch) -> TransfiguredTouchIterator<'a> {
        TransfiguredTouchIterator {
            transposition : change,
            touch : touch,

            next_bell_index : 0,
            next_ruleoff_index : 0
        }
    }
}

impl<'a> TouchIterator for TransfiguredTouchIterator<'a> {
    fn next_bell (&mut self) -> Option<Bell> {
        if self.next_bell_index >= self.touch.length * self.touch.stage.as_usize () {
            return None;
        }

        let bell = self.touch.bells [self.next_bell_index];

        self.next_bell_index += 1;

        Some (bell)
    }

    fn next_ruleoff (&mut self) -> Option<usize> {
        if self.next_ruleoff_index >= self.touch.ruleoffs.len () {
            return None;
        }

        let index = self.touch.ruleoffs [self.next_ruleoff_index];

        self.next_ruleoff_index += 1;

        Some (index)
    }
}

trait TouchIterator {
    fn next_bell (&mut self) -> Option<Bell>;
    fn next_ruleoff (&mut self) -> Option<usize>;
}







#[cfg(test)]
mod touch_tests {
    use crate::touch::Touch;
    use crate::transposition::Transposition;

    #[test]
    fn row_iterator () {
        for s in vec! [
            "123456\n214365\n123456",
            "123\n213\n231\n321\n312\n132\n123",
            "1"
        ] {
            let mut chars = s.chars ();
            let touch = Touch::from (s);

            for row in touch.row_iterator () {
                for b in row.slice () {
                    match chars.next () {
                        Some (c) => { assert_eq! (b.as_char (), c); }
                        None => { panic! ("Touch yielded too many bells"); }
                    }
                }

                chars.next (); // Consume the newlines
            }

            assert_eq! (chars.next (), None);
        }
    }

    #[test]
    fn string_conversions () {
        for s in vec! [
            "123456\n214365\n123456",
            "123\n213\n231\n321\n312\n132\n123",
            "1"
        ] {
            assert_eq! (Touch::from (s).to_string (), s);
        }
    }
}
