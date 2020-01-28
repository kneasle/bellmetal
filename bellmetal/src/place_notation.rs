use crate::types::*;
use crate::consts;
use crate::types::MaskMethods;
use crate::change::Change;
use std::cmp::PartialEq;
use std::fmt;

#[derive(Copy, Clone)]
pub struct PlaceNotation {
    pub places : Mask,
    pub stage : Stage
}

impl PartialEq for PlaceNotation {
    fn eq (&self, other : &Self) -> bool {
        self.places == other.places && self.stage == other.stage
    }
}

impl Eq for PlaceNotation { }

impl fmt::Debug for PlaceNotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity (Mask::limit () as usize);

        for i in 0..self.stage.as_usize () {
            if self.places.get (i as Number) {
                s.push (Bell::from (i).as_char ());
            }
        }

        if s.len () == 0 {
            s.push ('x');
        }

        write! (f, "{}", s)
    }
}

impl PlaceNotation {
    pub fn transposition (&self) -> Change {
        let stage = self.stage.as_usize ();
        let mut bell_vec : Vec<Bell> = Vec::with_capacity (stage);

        let mut i = 0;
        
        while i < stage {
            if self.places.get (i as u32) || self.places.get (i as u32 + 1) {
                bell_vec.push (Bell::from (i));

                i += 1;
            } else {
                bell_vec.push (Bell::from (i + 1));
                bell_vec.push (Bell::from (i));
                i += 2;
            }
        }

        Change { seq : bell_vec }
    }

    pub fn is_cross (notation : char) -> bool {
        notation == 'X' || notation == 'x' || notation == '-'
    }

    pub fn cross (stage : Stage) -> PlaceNotation {
        if stage.as_u32 () & 1u32 != 0 {
            panic! ("Non-even stage used with a cross notation");
        }

        PlaceNotation { places : Mask::empty (), stage : stage }
    }

    pub fn from_string (notation : &str, stage : Stage) -> PlaceNotation {
        let mut places = Mask::empty ();
        
        if notation == "X" || notation == "x" || notation == "-" {
            if stage.as_u32 () & 1u32 != 0 {
                panic! ("Non-even stage used with a cross notation");
            }
            
            // Nothing to be done here, since places defaults to 0
        } else { // Should decode bell names as places
            for c in notation.chars () {
                if !consts::is_bell_name (c) {
                    panic! ("Unknown bell name '{}' found in place notation '{}'", c, notation);
                }

                places.add (consts::name_to_number (c));
            }

            // Add implicit places (lower place)
            let mut lowest_place = 0 as Number;

            while !places.get (lowest_place) {
                lowest_place += 1;
            }

            if lowest_place & 1 == 1 {
                places.add (0 as Number);
            }

            // Add implicit places (higher place)
            let mut highest_place = stage.as_number ();

            while !places.get (highest_place) {
                highest_place -= 1;
            }
            
            if (stage.as_number () - highest_place) & 1 == 0 {
                places.add (stage.as_number () - 1);
            }
        }
        
        PlaceNotation { places : places, stage : stage }
    }

    pub fn from_multiple_string<'a> (string : &str, stage : Stage) -> Vec<PlaceNotation> {
        let mut string_buff = String::with_capacity (Mask::limit () as usize);
        let mut place_notations : Vec<PlaceNotation> = Vec::with_capacity (string.len ());
        let mut comma_index = 0usize;
        let mut has_found_comma = false;

        macro_rules! add_place_not {
            () => {
                place_notations.push (PlaceNotation::from_string (&string_buff, stage));
                string_buff.clear ();
            }
        }

        for c in string.chars () {
            if c == '.' || c == ' ' {
                add_place_not! ();
            } else if c == ',' {
                add_place_not! ();
                
                has_found_comma = true;
                comma_index = place_notations.len ();
            } else if PlaceNotation::is_cross (c) {
                if string_buff.len () != 0 {
                    add_place_not! ();
                }
                
                place_notations.push (PlaceNotation::cross (stage));
            } else {
                string_buff.push (c);
            }
        }

        add_place_not! ();

        // Deal with strings with comma in them
        if has_found_comma {
            println! (" >> {}", comma_index);

            let mut reordered_place_notations : Vec<PlaceNotation> = Vec::with_capacity (
                comma_index * 2 + (place_notations.len () - comma_index) * 2 - 2
            );

            macro_rules! add {
                ($x : expr) => {
                    reordered_place_notations.push (place_notations [$x].clone ());
                }
            }
            
            // Before the comma forwards
            for i in 0..comma_index {
                add! (i);
            }

            // Before the comma backwards
            for i in 0..comma_index - 1 {
                add! (comma_index - 2 - i);
            }
            
            // After the comma forwards
            for i in comma_index..place_notations.len () {
                add! (i);
            }

            // After the comma backwards
            for i in 0..place_notations.len () - comma_index - 1 {
                add! (place_notations.len () - 2 - i);
            }

            reordered_place_notations
        } else {
            place_notations
        }
    }
}




struct PlaceNotationIterator<'a> {
    place_notation : &'a PlaceNotation,
    index : Number,
    should_hunt_up : bool
}

impl <'a> Iterator for PlaceNotationIterator<'a> {
    type Item = Number;

    fn next (&mut self) -> Option<Number> {
        if self.index == self.place_notation.stage.as_number () {
            return None;
        }
        
        let mut output = 0 as Number;
    
        if self.place_notation.places.get (self.index) {
            output = self.index;

            self.should_hunt_up = false;
        } else {
            if self.should_hunt_up {
                output = self.index - 1;
            } else {
                output = self.index + 1;
            }
            
            self.should_hunt_up = !self.should_hunt_up;
        }

        self.index += 1;
        
        return Some (output);
    }
}




#[cfg(test)]
pub mod pn_tests {
    use crate::types::*;
    use crate::place_notation::PlaceNotation;
    use crate::change::{ Change, ChangeAccumulator };

    #[test]
    fn equality () {
        assert! (
            PlaceNotation::from_string ("14", Stage::MINIMUS)
            ==
            PlaceNotation::from_string ("14", Stage::MINIMUS)
        );
        
        assert! (
            PlaceNotation::from_string ("14", Stage::MINIMUS)
            !=
            PlaceNotation::from_string ("14", Stage::DOUBLES)
        );
        
        assert! (
            PlaceNotation::from_string ("14", Stage::MAJOR)
            !=
            PlaceNotation::from_string ("1458", Stage::MAJOR)
        );
    }

    #[test]
    fn implicit_places () {
        assert_eq! (
            PlaceNotation::from_string ("4", Stage::TRIPLES),
            PlaceNotation::from_string ("147", Stage::TRIPLES)
        );

        assert_eq! (
            PlaceNotation::from_string ("47", Stage::CATERS),
            PlaceNotation::from_string ("147", Stage::CATERS)
        );

        assert_eq! (
            PlaceNotation::from_string ("45", Stage::MAJOR),
            PlaceNotation::from_string ("1458", Stage::MAJOR)
        );
    }

    #[test]
    fn transpositions () {
        assert_eq! (
            PlaceNotation::from_string ("4", Stage::TRIPLES).transposition (),
            Change::from ("1324657")
        );
        
        assert_eq! (
            PlaceNotation::from_string ("x", Stage::MAJOR).transposition (),
            Change::from ("21436587")
        );
        
        assert_eq! (
            PlaceNotation::from_string ("135", Stage::DOUBLES).transposition (),
            Change::from ("12345")
        );
    }

    #[test]
    fn split_many_and_change_accum () {
        fn test (string : &str, stage : Stage, result : Change)  {
            let split_notation = PlaceNotation::from_multiple_string (string, stage);
            
            // Naive and extremely ineffecient accumulation
            let mut accum : Change = Change::rounds (stage);
            
            for c in &split_notation {
                accum = accum * c.transposition ();
            }

            assert_eq! (accum, result);

            // Much faster accumulation function
            let mut change_accum = ChangeAccumulator::new (stage);

            for c in split_notation {
                change_accum.accumulate (&c.transposition ()); // TODO: Implement an iterator conversion for Transposition
            }

            assert_eq! (*change_accum.total (), result);
        }

        test ("x16", Stage::MINOR, Change::from ("241635")); // Original Minor
        test ("3.145.5.1.5.1.5.1.5.1", Stage::DOUBLES, Change::from ("12435")); // Gnu Bob Doubles
        test ("3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", Stage::TRIPLES, Change::from ("4623751")); // Scientific Triples
        test ("x12,16", Stage::MINOR, Change::from ("142635")); // Bastow Minor
        test ("3,1.E.1.E.1.E.1.E.1.E.1", Stage::CINQUES, Change::from ("12537496E80")); // Grandsire Cinques
    }
}
