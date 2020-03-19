use crate::types::*;
use crate::consts;
use crate::{ Change, ChangeAccumulator, MaskMethods };
use std::cmp::PartialEq;
use std::fmt;

#[derive(Hash, Copy, Clone)]
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
    fn fmt (&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity (Mask::limit () as usize);

        self.into_string (&mut s);

        write! (f, "{}", s)
    }
}

impl PlaceNotation {
    pub fn is_cross (&self) -> bool {
        let mut count = 0;

        for i in 0..self.stage.as_usize () {
            if self.places.get (i as Number) {
                count += 1;
            }
        }

        count == 0
    }

    pub fn iterator (&self) -> PlaceNotationIterator {
        PlaceNotationIterator::new (self)
    }
    
    // Returns the place notation that represents 'self' but with the places reversed
    // (for example 14 -> 58 in Major, 1 -> 7 in Triples, etc)
    pub fn reversed (&self) -> PlaceNotation {
        let stage = self.stage.as_usize ();

        let mut places = Mask::empty ();
        
        for i in 0..stage {
            if self.places.get (i as Number) {
                places.add ((stage - i - 1) as Number);
            }
        }

        PlaceNotation { 
            places : places, 
            stage : self.stage
        }
    }

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

        Change::new (bell_vec)
    }
    
    pub fn into_string_implicit (&self, string : &mut String) {
        let mut count = 0;
        let mut is_1sts_made = false;
        let mut is_nths_made = false;
        let mut internal_place_count = 0;

        let stage = self.stage.as_usize ();

        for i in 0..stage { // Don't cover implicit places
            if self.places.get (i as Number) {
                if i == 0 {
                    is_1sts_made = true;
                } else if i == stage - 1 {
                    is_nths_made = true;
                } else {
                    internal_place_count += 1;

                    string.push (Bell::from (i).as_char ());
                }
                
                count += 1;
            }
        }

        if count == 0 {
            string.push ('x');
        } else {
            if internal_place_count > 0 {
                return;
            }

            if is_1sts_made {
                string.push (Bell::from (0).as_char ());
            } else if is_nths_made {
                string.push (Bell::from (stage - 1).as_char ());
            }
        }
    }

    pub fn into_string (&self, string : &mut String) {
        let mut count = 0;

        for i in 0..self.stage.as_usize () {
            if self.places.get (i as Number) {
                string.push (Bell::from (i).as_char ());
                
                count += 1;
            }
        }

        if count == 0 {
            string.push ('x');
        }
    }
}

impl PlaceNotation {
    pub fn is_cross_notation (notation : char) -> bool {
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
        
        if notation == "" || notation == "X" || notation == "x" || notation == "-" {
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

    pub fn into_multiple_string_short (place_notations : &Vec<PlaceNotation>, string : &mut String) {
        let len = place_notations.len ();

        let is_symmetrical = |i : usize| -> bool {
            for j in 0..i >> 1 {
                if place_notations [j] != place_notations [i - j - 1] {
                    return false;
                }
            }
            for j in 0..(len - i) >> 1 {
                if place_notations [i + j] != place_notations [len - j - 1] {
                    return false;
                }
            }

            true
        };

        // Decide on the location, if any, of the comma
        let mut comma_index : Option<usize> = None;
        
        if place_notations.len () % 2 == 0 {
            if is_symmetrical (len - 1) {
                comma_index = Some (len - 1);
            } else {
                for i in (1..len - 1).step_by (2) {
                    if is_symmetrical (i) {
                        comma_index = Some (i);
                        break;
                    }
                }
            }
        }

        // Generate string
        let mut was_last_place_notation_cross = true; // Used to decide whether to insert a dot
        
        match comma_index {
            Some (x) => {
                // Before comma
                for p in &place_notations [..x / 2 + 1] {
                    if p.is_cross () {
                        string.push ('x');

                        was_last_place_notation_cross = true;
                    } else {
                        if !was_last_place_notation_cross {
                            string.push ('.');
                        }

                        p.into_string_implicit (string);

                        was_last_place_notation_cross = false;
                    }
                }

                string.push (',');
                was_last_place_notation_cross = true;
                
                // After comma
                for p in &place_notations [x..x + (len - x) / 2 + 1] {
                    if p.is_cross () {
                        string.push ('x');

                        was_last_place_notation_cross = true;
                    } else {
                        if !was_last_place_notation_cross {
                            string.push ('.');
                        }

                        p.into_string_implicit (string);

                        was_last_place_notation_cross = false;
                    }
                }
            }
            None => {
                for p in place_notations {
                    if p.is_cross () {
                        string.push ('x');

                        was_last_place_notation_cross = true;
                    } else {
                        if !was_last_place_notation_cross {
                            string.push ('.');
                        }

                        p.into_string_implicit (string);

                        was_last_place_notation_cross = false;
                    }
                }
            }
        }
    }

    pub fn into_multiple_string (place_notations : &Vec<PlaceNotation>, string : &mut String) {
        let mut was_last_place_notation_cross = true; // Used to decide whether to insert a dot

        for p in place_notations {
            if p.is_cross () {
                string.push ('x');

                was_last_place_notation_cross = true;
            } else {
                if !was_last_place_notation_cross {
                    string.push ('.');
                }

                p.into_string (string);

                was_last_place_notation_cross = false;
            }
        }
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
            } else if PlaceNotation::is_cross_notation (c) {
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

    pub fn overall_transposition (pns : &[PlaceNotation]) -> Change {
        if pns.len () == 0 {
            panic! ("Can't find overall transposition of empty PlaceNotation list");
        }

        let mut accum = ChangeAccumulator::new (pns [0].stage);

        for pn in pns {
            accum.accumulate_iterator (pn.iterator ());
        }
        
        accum.total ().clone ()
    }
}




pub struct PlaceNotationIterator<'a> {
    place_notation : &'a PlaceNotation,
    index : usize,
    should_hunt_up : bool
}

impl PlaceNotationIterator<'_> {
    fn new (place_notation : &PlaceNotation) -> PlaceNotationIterator {
        PlaceNotationIterator {
            place_notation : place_notation,
            index : 0,
            should_hunt_up : false
        }
    }
}

impl <'a> Iterator for PlaceNotationIterator<'a> {
    type Item = Bell;

    fn next (&mut self) -> Option<Bell> {
        if self.index == self.place_notation.stage.as_usize () {
            return None;
        }
        
        #[allow(unused_assignments)]
        let mut output = 0;
    
        if self.place_notation.places.get (self.index as Number) {
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
        
        return Some (Bell::from (output));
    }
}




#[cfg(test)]
pub mod pn_tests {
    use crate::{
        Stage,
        PlaceNotation,
        Change, ChangeAccumulator
    };

    #[test]
    fn is_cross () {
        assert! (PlaceNotation::from_string ("x", Stage::MAXIMUS).is_cross ());
        assert! (PlaceNotation::from_string ("-", Stage::MAJOR).is_cross ());
        assert! (PlaceNotation::from_string ("X", Stage::MINOR).is_cross ());
        assert! (PlaceNotation::from_string ("", Stage::ROYAL).is_cross ());
        assert! (!PlaceNotation::from_string ("1", Stage::TRIPLES).is_cross ());
        assert! (!PlaceNotation::from_string ("18", Stage::MAJOR).is_cross ());
        assert! (!PlaceNotation::from_string ("3", Stage::SINGLES).is_cross ());
    }

    #[test]
    fn multiple_string_conversions () {
        let mut s = String::with_capacity (100);

        for (string, stage) in &[
            ("x16", Stage::MINOR), // Original Minor
            ("3.145.5.1.5.1.5.1.5.1", Stage::DOUBLES), // Gnu Bob Doubles
            ("3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", Stage::TRIPLES), // Scientific Triples
            ("x12x16", Stage::MINOR), // Bastow Minor
            ("3.1.E.1.E.1.E.1.E.1.E.1.E.1.E.1.E.1.E.1.E.1", Stage::CINQUES) // Grandsire Cinques
        ] {
            PlaceNotation::into_multiple_string (&PlaceNotation::from_multiple_string (string, *stage), &mut s);

            assert_eq! (s, *string);

            s.clear ();
        }
    }

    #[test]
    fn single_string_conversions () {
        let mut s = String::with_capacity (10);

        for (pn, stage, exp) in &[
            ("x", Stage::MAJOR, "x"),
            ("123", Stage::SINGLES, "123"),
            ("149", Stage::CINQUES, "149"),
            ("189", Stage::CATERS, "189"),
            ("45", Stage::MAJOR, "1458"),
            ("2", Stage::TRIPLES, "127"),
            ("", Stage::ROYAL, "x"),
            ("4", Stage::SIXTEEN, "14"),
        ] {
            PlaceNotation::from_string (pn, *stage).into_string (&mut s);
            
            assert_eq! (s, *exp);

            s.clear ();
        }
    }

    #[test]
    fn reversal () {
        for (original, reversed, stage) in &[
            ("x", "x", Stage::MINIMUS),
            ("147", "147", Stage::TRIPLES),
            ("1", "7", Stage::TRIPLES),
            ("14", "58", Stage::MAJOR),
            ("1490", "1270", Stage::ROYAL)
        ] {
            assert_eq! (
                PlaceNotation::from_string (original, *stage).reversed (),
                PlaceNotation::from_string (reversed, *stage)
            );
        }
    }

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
        for (lhs, rhs, stage) in &[
            ("4", "147", Stage::TRIPLES),
            ("47", "147", Stage::CATERS),
            ("45", "1458", Stage::MAJOR),
            ("1", "10", Stage::ROYAL)
        ] {
            assert_eq! (
                PlaceNotation::from_string (lhs, *stage),
                PlaceNotation::from_string (rhs, *stage)
            );
        }
    }

    #[test]
    fn transpositions () {
        for (lhs, rhs) in &[
            ("4", "1324657"),
            ("x", "2143658709"),
            ("x", "21436587"),
            ("135", "12345")
        ] {
            assert_eq! (
                PlaceNotation::from_string (lhs, Stage::from (rhs.len ())).transposition (),
                Change::from (*rhs)
            );
        }
    }

    #[test]
    fn implicit_places_removal () {
        let mut s = String::with_capacity (10);

        for (from, stage, to) in &[
            ("1", Stage::SINGLES, "1"),
            ("3", Stage::SINGLES, "3"),
            ("123", Stage::SINGLES, "2"),
            ("1", Stage::DOUBLES, "1"),
            ("3", Stage::DOUBLES, "3"),
            ("5", Stage::DOUBLES, "5"),
            ("125", Stage::DOUBLES, "2"),
            ("x", Stage::MINOR, "x"),
            ("14", Stage::MINOR, "4"),
            ("16", Stage::MINOR, "1"),
            ("1456", Stage::MINOR, "45"),
            ("14", Stage::SIXTEEN, "4"),
        ] {
            PlaceNotation::from_string (from, *stage).into_string_implicit (&mut s);

            println! ("{}", from);

            assert_eq! (s, *to);

            s.clear ();
        }
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

            for c in &split_notation {
                change_accum.accumulate_iterator (c.iterator ());
            }

            assert_eq! (*change_accum.total (), result);

            // Built-in accum function
            assert_eq! (PlaceNotation::overall_transposition (&split_notation), result);
        }

        test ("x16", Stage::MINOR, Change::from ("241635")); // Original Minor
        test ("3.145.5.1.5.1.5.1.5.1", Stage::DOUBLES, Change::from ("12435")); // Gnu Bob Doubles
        test ("3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", Stage::TRIPLES, Change::from ("4623751")); // Scientific Triples
        test ("x12,16", Stage::MINOR, Change::from ("142635")); // Bastow Minor
        test ("3,1.E.1.E.1.E.1.E.1.E.1", Stage::CINQUES, Change::from ("12537496E80")); // Grandsire Cinques
    }
}
