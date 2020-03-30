use crate::types::*;
use crate::Change;

pub trait Transposition {
    fn slice (&self) -> &[Bell];

    fn naive_hash (&self) -> usize {
        let mut val = 0;
        let stage = self.slice ().len ();

        for b in self.slice () {
            val *= stage;
            val += b.as_usize ();
        }

        val
    }

    fn stage (&self) -> Stage {
        Stage::from (self.slice ().len ())
    }

    fn bell_at (&self, place : Place) -> Bell {
        self.slice () [place.as_usize ()]
    }

    fn place_of (&self, bell : Bell) -> Place {
        for (i, b) in self.slice ().iter ().enumerate () {
            if *b == bell {
                return Place::from (i);
            }
        }

        panic! ("Bell '{}' not found in change {:?}", bell.as_char (), self.slice ());
    }

    fn parity (&self) -> Parity {
        let bells = self.slice ();
        let stage = bells.len ();

        let mut mask = Mask::empty ();
        let mut bells_fixed = 0;

        let mut total_cycle_length = 0;

        while bells_fixed < stage {
            let mut bell = 0;
                
            while mask.get (bell) {
                bell += 1;
            }

            total_cycle_length += 1; // Make sure that the parity is correct

            while !mask.get (bell) {
                mask.add (bell);
                
                bell = bells [bell as usize].as_number ();

                total_cycle_length += 1;
                bells_fixed += 1;
            }
        }
        
        match total_cycle_length & 1 {
            0 => { Parity::Even },
            1 => { Parity::Odd }
            _ => { panic! ("Unknown parity") }
        }
    }

    // Music scoring (follows roughly what CompLib does, but IMO it makes long runs overpowered)
    fn music_score (&self) -> usize {
        run_length_to_score (self.run_length_off_front ())
            + run_length_to_score (self.run_length_off_back ())
    }

    fn run_length_off_front (&self) -> usize {
        let bells = self.slice ();

        let stage = bells.len ();

        if stage <= 1 {
            return stage;
        }

        let mut last = bells [0].as_number ();
        let mut i = 1;

        while i < stage && (
            bells [i].as_i32 () - last as i32 == -1 
         || bells [i].as_i32 () - last as i32 == 1
        ) {
            last = bells [i].as_number ();

            i += 1;
        }
        
        i
    }

    fn run_length_off_back (&self) -> usize {
        let bells = self.slice ();

        let stage = bells.len ();

        if stage <= 1 {
            return stage;
        }

        let mut last = bells [stage - 1].as_number ();
        let mut i = 1;

        while i < stage && (
            bells [stage - 1 - i].as_i32 () - last as i32 == -1 
         || bells [stage - 1 - i].as_i32 () - last as i32 == 1
        ) {
            last = bells [stage - 1 - i].as_number ();

            i += 1;
        }
        
        i
    }

    // Useful change tests
    fn is_full_cyclic (&self) -> bool {
        let bells = self.slice ();

        let stage = bells.len ();

        if stage == 0 {
            return false;
        }

        let start = bells [0].as_usize ();

        for i in 0..stage {
            if bells [i].as_usize () != (start + i) % stage {
                return false;
            }
        }

        true
    }

    fn is_reverse_full_cyclic (&self) -> bool {
        let bells = self.slice ();

        let stage = bells.len ();

        if stage == 0 {
            return false;
        }

        let start = bells [0].as_usize () + stage;

        for i in 0..stage {
            if bells [i].as_usize () != (start - i) % stage {
                return false;
            }
        }

        true
    }

    fn is_fixed_treble_cyclic (&self) -> bool {
        let bells = self.slice ();

        let stage = bells.len ();
        
        if stage <= 2 || bells [0].as_usize () != 0 {
            return false;
        }

        let start = bells [1].as_usize ();

        for i in 0..stage - 1 {
            let expected_bell = if start + i >= stage { start + i - stage + 1 } else { start + i };
            
            if bells [i + 1].as_usize () != expected_bell {
                return false;
            }
        }

        true
    }

    fn is_reverse_fixed_treble_cyclic (&self) -> bool {
        // This works the same way is 'is_fixed_treble_cyclic', but it iterates backwards
        // starting with the bell at the back

        let bells = self.slice ();

        let stage = bells.len ();
        
        if stage <= 2 || bells [0].as_usize () != 0 {
            return false;
        }

        let start = bells [stage - 1].as_usize ();

        for i in 0..stage - 1 {
            let expected_bell = if start + i >= stage { start + i - stage + 1 } else { start + i };

            if bells [stage - 1 - i].as_usize () != expected_bell {
                return false;
            }
        }

        true
    }

    fn is_backrounds (&self) -> bool {
        let bells = self.slice ();
        let stage = bells.len ();

        for i in 0..stage {
            if bells [i].as_usize () != stage - 1 - i {
                return false;
            }
        }

        true
    }

    fn is_rounds (&self) -> bool {
        let bells = self.slice ();
        let stage = bells.len ();

        for i in 0..stage {
            if bells [i].as_usize () != i {
                return false;
            }
        }

        true
    }

    fn reflected (&self) -> Change where Self : std::marker::Sized {
        let stage_minus_1 = self.slice ().len () - 1;

        Change::new (
            self.slice ()
                .iter ()
                .rev ()
                .map (|b| Bell::from (stage_minus_1 - b.as_usize ()))
                .collect ()
        )
    }

    // To string
    fn to_string (&self) -> String {
        let mut string = String::with_capacity (self.slice ().len ());

        for i in self.slice () {
            string.push (i.as_char ());
        }

        string
    }

    // Pretty printing
    fn pretty_string (&self) -> String {
        let mut string = String::with_capacity (self.slice ().len () * 3); // Seems a good length
        
        self.write_pretty_string (false, &mut string);

        string
    }

    fn write_pretty_string (&self, underline : bool, string : &mut String) {
        let bells = self.slice ();

        let stage = bells.len ();

        let run_front = {
            let x = self.run_length_off_front ();

            if x >= 4 { x } else { 0 }
        };

        let run_back = {
            let x = self.run_length_off_back ();

            if x >= 4 { x } else { 0 }
        };

        let mut was_last_char_highlighted = false;
        let mut last_char_colour = 0;

        let colours = ["97", "91", "96"];

        for i in 0..stage {
            // Useful vars
            let bell = bells [i];

            let char_colour = if bell.as_usize () == 0 {
                1
            } else if bell.as_usize () == stage - 1 {
                2
            } else {
                0
            };
            
            let should_be_highlighted = i < run_front || (stage - 1 - i) < run_back;
            
            // Push the escape codes
            if last_char_colour != char_colour || was_last_char_highlighted != should_be_highlighted {
                string.push_str ("\x1b[");
                if underline {
                    string.push_str ("4;");
                }
                string.push_str (colours [char_colour]);
                string.push (';');
                string.push_str (if should_be_highlighted { "42" } else { "49" });
                string.push ('m');
            }
            
            string.push (bell.as_char ());

            was_last_char_highlighted = should_be_highlighted;
            last_char_colour = char_colour;
        }

        string.push_str ("\x1b[0m"); // Always reset the formatting
    }
}







pub struct TranspositionIterator<'a> {
    slice : &'a [Bell],
    index : usize
}

impl TranspositionIterator<'_> {
    pub fn from_slice<'a> (slice : &'a [Bell]) -> TranspositionIterator<'a> {
        TranspositionIterator {
            slice : slice,
            index : 0
        }
    }

    pub fn from_transposition<'a> (transposition : &'a impl Transposition) -> TranspositionIterator<'a> {
        TranspositionIterator::from_slice (transposition.slice ())
    }
}

impl Iterator for TranspositionIterator<'_> {
    type Item = Bell;

    fn next (&mut self) -> Option<Bell> {
        if self.index >= self.slice.len () {
            return None;
        }

        let bell = Bell::from (self.slice [self.index]);

        self.index += 1;

        Some (bell)
    }
}






pub struct MultiplicationIterator<'a> {
    lhs : &'a [Bell],
    rhs : TranspositionIterator<'a>
}

impl MultiplicationIterator<'_> {
    pub fn new<'a> (lhs : &'a [Bell], rhs : TranspositionIterator<'a>) -> MultiplicationIterator<'a> {
        MultiplicationIterator {
            lhs : lhs,
            rhs : rhs
        }
    }
}

impl<'a> Iterator for MultiplicationIterator<'a> {
    type Item = Bell;

    fn next (&mut self) -> Option<Bell> {
        match self.rhs.next () {
            Some (b) => { Some (self.lhs [b.as_usize ()]) }
            None => { None }
        }
    }
}






fn run_length_to_score (length : usize) -> usize {
    if length < 4 {
        return 0;
    }

    let x = length - 3;
    
    // Triangular numbers = n * (n + 1) / 2
    (x * (x + 1)) >> 1
}
