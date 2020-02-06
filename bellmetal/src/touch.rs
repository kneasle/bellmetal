use crate::types::{ Stage, Bell };
use crate::place_notation::PlaceNotation;
use crate::change::Change;

pub struct Row<'a> {
    index : usize,
    bells : &'a [Bell]
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
    fn get_row_at<'a> (&'a self, index : usize) -> Row<'a> {
        let stage = self.stage.as_usize ();

        Row {
            index : index,
            bells : &self.bells [index * stage .. (index + 1) * stage]
        }
    }

    fn leftover_change (&self) -> Change {
        let stage = self.stage.as_usize ();

        let mut seq : Vec<Bell> = Vec::with_capacity (stage);
        let start = (self.length + 1) * stage;

        for i in 0..stage {
            seq.push (self.bells [start + i]);
        }
        
        Change::new (seq)
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

impl From<&Vec<PlaceNotation>> for Touch {
    fn from (place_notations : &Vec<PlaceNotation>) -> Touch {
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
            
            let mut change = Change::rounds (Stage::from (stage));
            
            macro_rules! add_change {
                () => {
                    for b in change.iterator (){
                        bells.push (b);
                    }
                }
            }
            
            // This will cause a lot of heap allocations, but I don't expect it will be called
            // a lot - however if this function is a bottleneck, then this might be a good
            // place to optimise
            
            add_change! ();
            for p in place_notations {
                change = change * p.transposition ();
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

#[cfg(test)]
mod touch_tests {
    use crate::touch::Touch;

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
