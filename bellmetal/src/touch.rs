use crate::types::{ Stage, Bell, Place };
use crate::place_notation::PlaceNotation;
use crate::change::{ Change, ChangeAccumulator };
use crate::transposition::Transposition;

use std::cmp::Ordering;

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

    bells : Vec<Bell>,
    ruleoffs : Vec<usize>,
    pub leftover_change : Change
}

impl Touch {
    pub fn row_iterator<'a> (&'a self) -> RowIterator<'a> {
        RowIterator::new (self)
    }

    pub fn row_at (&self, index : usize) -> Row {
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

    pub fn bell_at (&self, index : usize) -> Bell {
        self.bells [index]
    }

    pub fn music_score (&self) -> usize {
        let mut music_score = 0;

        for r in self.row_iterator () {
            music_score += r.music_score ();
        }

        music_score
    }

    pub fn is_true (&self) -> bool {
        let mut rows : Vec<Row> = self.row_iterator ().collect ();
        let stage = self.stage.as_usize ();

        rows.sort_by (
            |a, b| {
                let mut i = 0;

                loop {
                    if i == stage {
                        return Ordering::Equal;
                    }

                    if a.bells [i] == b.bells [i] {
                        i += 1;
                    } else if a.bells [i] < b.bells [i] {
                        return Ordering::Less;
                    } else {
                        return Ordering::Greater;
                    }
                }
            }
        );

        for i in 1..rows.len () {
            let mut are_equal = true;
            
            let a = &rows [i - 1];
            let b = &rows [i];

            for p in 0..stage {
                if a.bells [p] != b.bells [p] {
                    are_equal = false;

                    break;
                }
            }

            if are_equal {
                return false;
            }
        }

        true
    }

    pub fn pretty_string_multi_column (&self, columns : usize) -> String {
        let stage = self.stage.as_usize ();
        let rows_per_column = self.length / columns;

        let mut lines : Vec<String> = Vec::with_capacity (rows_per_column * 2 + 1);
        
        let mut row_number : usize = 0;
        let mut line_number : usize = 0;

        for r in self.row_iterator () {
            // Start new column if required, and add the row to the bottom of the last column
            if row_number == rows_per_column {
                if line_number == lines.len () {
                    lines.push (String::with_capacity (200));
                } else {
                    lines [line_number].push_str ("    ");
                }

                r.write_pretty_string (false, &mut lines [line_number]);
                
                line_number = 0;
                row_number = 0;
            }
            
            // Push the row
            if line_number == lines.len () {
                lines.push (String::with_capacity (200));
            } else {
                lines [line_number].push_str ("    ");
            }

            r.write_pretty_string (false, &mut lines [line_number]);
            
            line_number += 1;
            
            // Push the ruleoff
            if r.is_ruled_off {
                if line_number == lines.len () {
                    lines.push (String::with_capacity (200));
                } else {
                    lines [line_number].push_str ("    ");
                }

                for _ in 0..stage {
                    lines [line_number].push ('-');
                }
                
                line_number += 1;
            }
            
            // Update the row_counter
            row_number += 1;
        }

        // Add the leftover change
        if line_number == lines.len () {
            lines.push (String::with_capacity (200));
        } else {
            lines [line_number].push_str ("    ");
        }

        self.leftover_change.write_pretty_string (false, &mut lines [line_number]);

        lines.join ("\n")
    }

    pub fn pretty_string (&self) -> String {
        let stage = self.stage.as_usize ();

        let mut s = String::with_capacity (stage * self.length * 2);

        for r in self.row_iterator () {
            r.write_pretty_string (false, &mut s);

            if r.is_ruled_off {
                s.push ('\n');

                for _ in 0..stage {
                    s.push ('-');
                }
            }

            s.push ('\n');
        }

        s
    }

    pub fn to_string (&self) -> String {
        let stage = self.stage.as_usize ();

        let mut s = String::with_capacity (stage * self.length + self.length);

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

impl Touch {
    pub fn from_iterator_multipart<I> (iterator : &mut I, part_ends : &[Change]) -> Touch 
            where I : TouchIterator, I : Sized {
        let num_parts = part_ends.len ();
        
        let stage = iterator.stage ().as_usize ();
        let length = iterator.length () * num_parts;
        
        let mut bells : Vec<Bell> = Vec::with_capacity (length * stage);
        let mut ruleoffs : Vec<usize> = Vec::with_capacity (iterator.number_of_ruleoffs () * num_parts);

        let mut part_start_index = 0;
        
        for c in part_ends {
            iterator.reset ();
            
            loop {
                match iterator.next_bell () {
                    Some (b) => { bells.push (c.bell_at (Place::from (b.as_usize ()))); }
                    None => { break; }
                }
            }

            loop {
                match iterator.next_ruleoff () {
                    Some (r) => { ruleoffs.push (r + part_start_index); }
                    None => { break; }
                }
            }

            part_start_index += iterator.length ();
        }

        Touch {
            stage : Stage::from (stage),
            length : length,
            
            bells : bells,
            ruleoffs : ruleoffs,
            
            leftover_change : Change::rounds (Stage::from (stage))
        }
    }

    pub fn from_iterator<I> (iterator : &mut I) -> Touch where I : TouchIterator, I : Sized {
        let stage = iterator.stage ().as_usize ();
        let length = iterator.length ();
        
        // Generate bells
        let mut bells : Vec<Bell> = Vec::with_capacity (length * stage);
        
        loop {
            match iterator.next_bell () {
                Some (b) => { bells.push (b); }
                None => { break; }
            }
        }

        // Generate ruleoffs
        let mut ruleoffs : Vec<usize> = Vec::with_capacity (iterator.number_of_ruleoffs ());

        loop {
            match iterator.next_ruleoff () {
                Some (r) => { ruleoffs.push (r); }
                None => { break; }
            }
        }

        Touch {
            stage : Stage::from (stage),
            length : length,
            
            bells : bells,
            ruleoffs : ruleoffs,
            
            leftover_change : iterator.leftover_change ()
        }
    }
}

impl From<&[PlaceNotation]> for Touch {
    fn from (place_notations : &[PlaceNotation]) -> Touch {
        let length = place_notations.len ();

        if length == 0 {
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
            
        let mut bells : Vec<Bell> = Vec::with_capacity (length * stage);
        let mut accumulator : ChangeAccumulator = ChangeAccumulator::new (Stage::from (stage));

        for p in place_notations {
            for b in accumulator.total ().iterator () {
                bells.push (b);
            }
            
            accumulator.accumulate_iterator (p.iterator ());
        }
        
        Touch {
            stage : Stage::from (stage),
            length : length,

            bells : bells,
            ruleoffs : Vec::with_capacity (0),
            leftover_change : accumulator.total ().clone ()
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
                Some (s) => { (s, length - 1) } // The last line will be the leftover_change
                None => { panic! ("Cannot create an empty touch"); }
            }
        };
        
        let mut bells : Vec<Bell> = Vec::with_capacity (length * stage);
        let mut leftover_vec : Vec<Bell> = Vec::with_capacity (stage);
        let mut counter = 0;

        for line in string.lines () {
            if counter < length {
                for c in line.chars () {
                    bells.push (Bell::from (c));
                }
            } else {
                for c in line.chars () {
                    leftover_vec.push (Bell::from (c));
                }
            }

            counter += 1;
        }

        Touch {
            stage : Stage::from (stage),
            length : length,

            bells : bells,
            ruleoffs : Vec::with_capacity (0),
            leftover_change : Change::new (leftover_vec)
        }
    }
}






pub struct RowIterator<'a> {
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





pub trait TouchIterator {
    fn next_bell (&mut self) -> Option<Bell>;
    fn next_ruleoff (&mut self) -> Option<usize>;
    fn reset (&mut self);

    fn length (&self) -> usize;
    fn stage (&self) -> Stage;

    fn number_of_ruleoffs (&self) -> usize;

    fn leftover_change (&self) -> Change;
}






struct TransfiguredTouchIterator<'a> {
    start_change : &'a Change,
    touch : &'a Touch,

    next_bell_index : usize,
    next_ruleoff_index : usize
}

impl TransfiguredTouchIterator<'_> {
    pub fn new<'a> (change : &'a Change, touch : &'a Touch) -> TransfiguredTouchIterator<'a> {
        TransfiguredTouchIterator {
            start_change : change,
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

        let bell = self.start_change.bell_at (Place::from (self.touch.bells [self.next_bell_index].as_usize ()));

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

    fn reset (&mut self) {
        self.next_bell_index = 0;
        self.next_ruleoff_index = 0;
    }

    fn length (&self) -> usize {
        self.touch.length
    }

    fn number_of_ruleoffs (&self) -> usize {
        self.touch.ruleoffs.len ()
    }

    fn stage (&self) -> Stage {
        self.touch.stage
    }

    fn leftover_change (&self) -> Change {
        self.start_change.multiply (&self.touch.leftover_change)
    }
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
            "1",
            "12345678
21436587
12346857
21438675
24136857
42316587
24135678
42315768
24351786
42537168
45231786
54327168
45237618
54326781
45362718
54637281
56473821
65748312
56784321
65873412
56783142
65871324
68573142
86751324
68715342
86175432
68714523
86174253
81672435
18764253
81674523
18765432
17856342" // First lead of Deva
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
            
            // Consume the leftover change
            for b in touch.leftover_change.iterator () {
                match chars.next () {
                    Some (c) => { assert_eq! (b.as_char (), c); }
                    None => { panic! ("Touch yielded too many bells"); }
                }
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
            let t = Touch::from (s);
            
            if t.to_string () != "" {
                assert_eq! (t.to_string () + "\n" + &t.leftover_change.to_string (), s);
            }
        }
    }
}
