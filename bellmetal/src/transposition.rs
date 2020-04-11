use crate::types::*;
use crate::Change;

pub trait Transposition {
    fn slice (&self) -> &[Bell];

    fn iter<'a> (&'a self) -> std::iter::Cloned<std::slice::Iter<'a, Bell>> {
        self.slice ().iter ().cloned ()
    }

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

        panic! (
            "Bell '{}' not found in <{}>", 
            bell.as_char (), 
            self.slice ()
                .iter ()
                .map (|x| x.as_char ())
                .collect::<String> ()
        );
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

    fn is_continuous_with<T : Transposition> (&self, other : T) -> bool {
        let a = self.slice ();
        let b = other.slice ();

        if a.len () != b.len () {
            return false;
        }

        let mut i = 0;

        while i < a.len () {
            if a [i] == b [i] {
                i += 1;
            } else if a [i] == b [i + 1] && b [i] == a [i + 1] {
                i += 2;
            } else {
                return false;
            }
        }

        true
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

    fn reflected (&self) -> Change where Self : Sized {
        let stage_minus_1 = self.slice ().len () - 1;

        Change::new (
            self.slice ()
                .iter ()
                .rev ()
                .map (|b| Bell::from (stage_minus_1 - b.as_usize ()))
                .collect ()
        )
    }

    fn copy_into (&self, other : &mut Change) where Self : Sized {
        other.overwrite_from (self);
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
        
        self.write_pretty_string (&mut string);

        string
    }

    fn write_pretty_string (&self, string : &mut String) {
        self.write_pretty_string_with_stroke (string, Stroke::Hand);
    }

    fn pretty_string_with_stroke (&self, stroke : Stroke) -> String {
        let mut string = String::with_capacity (self.slice ().len () * 3); // Seems a good length
        
        self.write_pretty_string_with_stroke (&mut string, stroke);

        string
    }

    fn write_pretty_string_with_stroke (&self, string : &mut String, stroke : Stroke) {
        #[derive(PartialEq, Eq)]
        enum CharState {
            Normal,
            Musical,
            Undesirable
        }

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

        let is_87_at_back = stage % 2 == 0 
            && stroke == Stroke::Back
            && bells [stage - 2] == Bell::from (stage - 1) 
            && bells [stage - 1] == Bell::from (stage - 2);

        let mut last_char_state = CharState::Normal;
        let mut last_char_colour = 0;

        let colours = ["97", "91", "96"];

        for i in 0..stage {
            // Useful vars
            let bell = bells [i];

            let char_colour = if bell.as_usize () == 0 { 1 } 
                    else if bell.as_usize () == stage - 1 { 2 } 
                    else { 0 };
            
            let char_state = if i < run_front || (stage - 1 - i) < run_back { CharState::Musical }
                    else if is_87_at_back && i >= stage - 2 { CharState::Undesirable }
                    else { CharState::Normal };
            
            // Push the escape codes
            if last_char_colour != char_colour || last_char_state != char_state {
                string.push_str ("\x1b[");
                string.push_str (colours [char_colour]);
                string.push (';');
                string.push_str (match char_state {
                    CharState::Musical => { "42" } 
                    CharState::Undesirable => { "41" } 
                    CharState::Normal => { "49" }
                });
                string.push ('m');
            }
            
            string.push (bell.as_char ());

            last_char_state = char_state;
            last_char_colour = char_colour;
        }

        string.push_str ("\x1b[0m"); // Always reset the formatting
    }
}






pub struct MultiplicationIterator<'a, T : Iterator<Item = Bell>> {
    lhs : &'a [Bell],
    rhs : T
}

impl<T> MultiplicationIterator<'_, T> where T : Iterator<Item = Bell> {
    pub fn new<'a> (lhs : &'a [Bell], rhs : T) -> MultiplicationIterator<'a, T> {
        MultiplicationIterator {
            lhs : lhs,
            rhs : rhs
        }
    }
}

impl<'a, T> Iterator for MultiplicationIterator<'a, T> where T : Iterator<Item = Bell> {
    type Item = Bell;

    fn next (&mut self) -> Option<Bell> {
        match self.rhs.next () {
            Some (b) => { Some (self.lhs [b.as_usize ()]) }
            None => { None }
        }
    }

    fn size_hint (&self) -> (usize, Option<usize>) {
        self.rhs.size_hint ()
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
