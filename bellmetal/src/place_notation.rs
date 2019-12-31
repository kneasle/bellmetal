use crate::types::*;
use crate::consts;
use crate::types::MaskMethods;

pub struct PlaceNotation<'a> {
    notation : &'a str,
    places : Mask,
    stage : Stage
}

pub fn generate (notation : &str, stage : Stage) -> PlaceNotation {
    let mut places = Mask::empty ();
    
    if notation == "X" || notation == "x" || notation == "" {
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
    
    PlaceNotation { notation : notation, places : places, stage : stage }
}

pub fn split (string : &str, stage : Stage) -> Vec<PlaceNotation> {
    let mut string_buff = String::with_capacity (Mask::limit () as usize);
    let mut places_mask = Mask::empty ();

    for c in string.chars () {
        print! ("{}", c);
    }

    vec![]
}

#[cfg(test)]
pub mod pn_tests {
    #[test]
    fn equality () {

    }
}
