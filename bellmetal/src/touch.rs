use crate::{
    Stage, Bell, Place, Stroke,
    PlaceNotation,
    Change, ChangeAccumulator,
    Transposition,
    NaiveProver, ProvingContext, FullProvingContext,
    Method
};

use std::cmp::Ordering;
use std::collections::HashMap;

fn falseness_to_table (falseness_map : &Vec<Vec<usize>>) -> HashMap<usize, usize> {
    let mut hash_map : HashMap<usize, usize> = HashMap::with_capacity (50);

    for (i, g) in falseness_map.iter ().enumerate () {
        for b in g {
            hash_map.insert (*b, i);
        }
    }

    hash_map
}



pub struct Row<'a> {
    pub index : usize,
    pub is_ruled_off : bool,
    pub call_char : char,
    pub stroke : Stroke,
    bells : &'a [Bell]
}



static MULTICOLUMN_DELIMITER : &str = "  ";
static ANNOTATION_PADDING_LEFT : &str = "    ";
static ANNOTATION_PADDING_RIGHT : &str = "  ";
static FALSENESS_COLOURS : [&str; 14] = [
    "91;1", "92;1", "93;1", "94;1", "95;1", "96;1", "97;1",
    "31", "32", "33", "34", "35", "36", "37"
];

impl Row<'_> {
    fn write_annotated_string (&self, string : &mut String, table : &HashMap<usize, usize>) {
        string.push_str (" ");
        string.push (self.call_char);
        
        match table.get (&self.index) {
            Some (x) => {
                string.push_str ("\x1b[");
                string.push_str (FALSENESS_COLOURS [*x % FALSENESS_COLOURS.len ()]);
                string.push_str ("m[\x1b[0m");
            }
            None => {
                string.push (' ');
            }
        }

        string.push (' ');
        
        self.write_pretty_string (string);
        
        string.push (' ');
        
        match table.get (&self.index) {
            Some (x) => {
                string.push_str ("\x1b[");
                string.push_str (FALSENESS_COLOURS [*x % FALSENESS_COLOURS.len ()]);
                string.push_str ("m]\x1b[0m");
            }
            None => {
                string.push (' ');
            }
        }
    }
}

impl Transposition for Row<'_> {
    fn slice (&self) -> &[Bell] {
        self.bells
    }
}

impl<'a> PartialEq for Row<'a> {
    fn eq (&self, other : &Self) -> bool {
        if self.stage () != other.stage () {
            return false;
        }

        for p in 0..self.stage ().as_usize () {
            if self.bells [p] != other.bells [p] {
                return false;
            }
        }

        true
    }
}

impl<'a> Eq for Row<'a> { }

impl<'a> Ord for Row<'a> {
    fn cmp (&self, other : &Self) -> Ordering {
        assert_eq! (self.stage (), other.stage ());

        let stage = self.stage ().as_usize ();
        let mut i = 0;

        loop {
            if i == stage {
                return Ordering::Equal;
            }

            if self.bells [i] == other.bells [i] {
                i += 1;
            } else if self.bells [i] < other.bells [i] {
                return Ordering::Less;
            } else {
                return Ordering::Greater;
            }
        }
    }
}

impl<'a> PartialOrd for Row<'a> {
    fn partial_cmp (&self, other : &Self) -> Option<Ordering> {
        Some (self.cmp (other))
    }
}





#[derive(PartialEq, Debug)]
pub struct Touch {
    pub stage : Stage,
    pub length : usize,

    bells : Vec<Bell>,
    ruleoffs : Vec<usize>,
    calls : HashMap<usize, char>,
    pub leftover_change : Change
}

impl Touch {
    pub fn row_iterator<'a> (&'a self) -> RowIterator<'a> {
        RowIterator::new (self)
    }

    pub fn iterator<'a> (&'a self) -> BasicTouchIterator<'a> {
        BasicTouchIterator::new (self)
    }

    pub fn add_call (&mut self, index : usize, call_char : char) {
        self.calls.insert (index, call_char);
    }

    pub fn append_iterator<'a> (&'a mut self, iterator : &mut impl TouchIterator) {
        iterator.reset ();

        loop {
            match iterator.next_bell () {
                Some (b) => { 
                    self.bells.push (b);
                }
                None => {
                    break;
                }
            }
        }

        loop {
            match iterator.next_ruleoff () {
                Some (b) => { 
                    self.ruleoffs.push (self.length + b);
                }
                None => {
                    break;
                }
            }
        }

        self.length += iterator.length ();
        
        self.ruleoffs.push (self.length - 1);
    }

    pub fn row_at (&self, index : usize) -> Row {
        let stage = self.stage.as_usize ();

        Row {
            index : index,
            is_ruled_off : match self.ruleoffs.binary_search (&index) {
                Ok (_) => { true }
                Err (_) => { false }
            },
            call_char : match self.calls.get (&index) {
                Some (c) => { *c }
                None => { ' ' }
            },
            stroke : Stroke::from_index (index),
            bells : &self.bells [index * stage .. (index + 1) * stage]
        }
    }

    pub fn slice_at (&self, index : usize) -> &[Bell] {
        let stage = self.stage.as_usize ();

        &self.bells [index * stage .. (index + 1) * stage]
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

    pub fn number_of_4_bell_runs (&self) -> (usize, usize) {
        let mut run_count_front = 0;
        let mut run_count_back = 0;

        for r in self.row_iterator () {
            if r.run_length_off_front () >= 4 {
                run_count_front += 1;
            }
            if r.run_length_off_back () >= 4 {
                run_count_back += 1;
            }
        }

        (run_count_front, run_count_back)
    }

    pub fn is_true (&self) -> bool {
        NaiveProver { }.prove (self)
    }

    pub fn full_truth (&self) -> Vec<Vec<usize>> {
        NaiveProver { }.full_prove (self)
    }

    pub fn full_truth_table (&self) -> HashMap<usize, usize> {
        falseness_to_table (&self.full_truth ())
    }

    pub fn pretty_string_multi_column (&self, columns : usize, truth : Option<&Vec<Vec<usize>>>) -> String {
        let truth_table = match truth {
            Some (t) => {
                falseness_to_table (t)
            }
            None => {
                self.full_truth_table ()
            }
        };

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
                    lines [line_number].push_str (MULTICOLUMN_DELIMITER);
                }

                r.write_annotated_string (&mut lines [line_number], &truth_table);
                
                line_number = 0;
                row_number = 0;
            }
            
            // Push the row
            if line_number == lines.len () {
                lines.push (String::with_capacity (200));
            } else {
                lines [line_number].push_str (MULTICOLUMN_DELIMITER);
            }

            r.write_annotated_string (&mut lines [line_number], &truth_table);
            
            line_number += 1;
            
            // Push the ruleoff
            if r.is_ruled_off {
                if line_number == lines.len () {
                    lines.push (String::with_capacity (200));
                } else {
                    lines [line_number].push_str (MULTICOLUMN_DELIMITER);
                }

                lines [line_number].push_str (ANNOTATION_PADDING_LEFT);

                for _ in 0..stage {
                    lines [line_number].push ('-');
                }
                
                lines [line_number].push_str (ANNOTATION_PADDING_RIGHT);

                line_number += 1;
            }
            
            // Update the row_counter
            row_number += 1;
        }

        // Add the leftover change
        if line_number == lines.len () {
            lines.push (String::with_capacity (200));
        } else {
            lines [line_number].push_str (MULTICOLUMN_DELIMITER);
        }

        lines [line_number].push_str (ANNOTATION_PADDING_LEFT);

        self.leftover_change.write_pretty_string (&mut lines [line_number]);

        lines.join ("\n")
    }

    pub fn pretty_string (&self, truth : Option<&Vec<Vec<usize>>>) -> String {
        let truth_table = match truth {
            Some (t) => {
                falseness_to_table (t)
            }
            None => {
                self.full_truth_table ()
            }
        };

        let stage = self.stage.as_usize ();

        let mut s = String::with_capacity (stage * self.length * 2);

        for r in self.row_iterator () {
            r.write_annotated_string (&mut s, &truth_table);

            if r.is_ruled_off {
                s.push ('\n');

                s.push_str (ANNOTATION_PADDING_LEFT);

                for _ in 0..stage {
                    s.push ('-');
                }
            }

            s.push ('\n');
        }

        s.push_str (ANNOTATION_PADDING_LEFT);

        self.leftover_change.write_pretty_string (&mut s);

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

    // Functions defined to increase performance by avoiding memory allocations
    pub fn overwrite_from_place_notations (&mut self, place_notations : &[PlaceNotation]) {
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
       
        // Bells
        self.bells.clear ();
        self.bells.reserve (length * stage);

        let mut accumulator : ChangeAccumulator = ChangeAccumulator::new (Stage::from (stage));

        for p in place_notations {
            for b in accumulator.total ().iterator () {
                self.bells.push (b);
            }
            
            accumulator.accumulate_iterator (p.iterator ());
        }
        
        // Ruleoffs
        self.ruleoffs.clear ();
        
        // Constants
        self.stage = Stage::from (stage);
        self.length = length;

        accumulator.total ().copy_into (&mut self.leftover_change);
    }

    pub fn overwrite_from_string (&mut self, string : &str) {
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
        
        // Constants
        self.length = length;
        self.stage = Stage::from (stage);
        
        self.bells.clear ();
        self.bells.reserve (length * stage);

        let mut counter = 0;

        for line in string.lines () {
            if counter < length {
                for c in line.chars () {
                    self.bells.push (Bell::from (c));
                }
            } else {
                self.leftover_change.overwrite_from_string (line);
            }

            counter += 1;
        }
    }

    pub fn reflected (&self) -> Touch {
        let mut new_bells : Vec<Bell> = Vec::with_capacity (self.length);
        let stage = self.stage.as_usize ();

        for r in self.row_iterator () {
            for b in r.slice ().iter ().rev () {
                new_bells.push (Bell::from (stage - 1 - b.as_usize ()));
            }
        }

        Touch {
            stage : self.stage,
            length : self.length,

            bells : new_bells,
            ruleoffs : self.ruleoffs.clone (),
            calls : self.calls.clone (),
            leftover_change : self.leftover_change.reflected ()
        }
    }
}

impl Touch {
    pub fn empty () -> Touch {
        Touch {
            stage : Stage::ZERO,
            length : 0usize,

            bells : Vec::with_capacity (0),
            ruleoffs : Vec::with_capacity (0),
            calls : HashMap::with_capacity (0),
            leftover_change : Change::empty ()
        }
    }

    pub fn with_capacity (stage : Stage, change_capacity : usize, ruleoff_capacity : usize, call_capacity : usize) -> Touch {
        Touch {
            stage : stage,
            length : 0usize,

            bells : Vec::with_capacity (change_capacity * stage.as_usize ()),
            ruleoffs : Vec::with_capacity (ruleoff_capacity),
            calls : HashMap::with_capacity (call_capacity),
            leftover_change : Change::rounds (stage)
        }
    }

    pub fn single_course (method : &Method, course_head : &Change) -> Touch {
        let mut accumulator = ChangeAccumulator::new (method.stage);
        let mut touch = Touch::empty ();

        accumulator.set (course_head);
        touch.stage = method.stage;

        loop {
            touch.append_iterator (&mut TransfiguredTouchIterator::new (accumulator.total (), &method.plain_lead));

            accumulator.accumulate (method.lead_head ());

            if accumulator.total () == course_head {
                break;
            }
        }

        touch
    }

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
            calls : HashMap::with_capacity (0),
            
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
            calls : HashMap::with_capacity (0),
            
            leftover_change : iterator.leftover_change ()
        }
    }
}

impl From<&[PlaceNotation]> for Touch {
    fn from (place_notations : &[PlaceNotation]) -> Touch {
        let mut touch = Touch::empty ();

        touch.overwrite_from_place_notations (place_notations);

        touch
    }
}

impl From<&str> for Touch {
    fn from (string : &str) -> Touch {
        let mut touch = Touch::empty ();

        touch.overwrite_from_string (string);

        touch
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
        let index = self.row_index;

        if index < self.touch.length {
            let row = self.touch.row_at (self.row_index);

            self.row_index += 1;
            if row.is_ruled_off {
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





pub struct BasicTouchIterator<'a> {
    touch : &'a Touch,

    next_bell_index : usize,
    next_ruleoff_index : usize
}

impl BasicTouchIterator<'_> {
    pub fn new<'a> (touch : &'a Touch) -> BasicTouchIterator<'a> {
        BasicTouchIterator {
            touch : touch,

            next_bell_index : 0,
            next_ruleoff_index : 0
        }
    }
}

impl<'a> TouchIterator for BasicTouchIterator<'a> {
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
        self.touch.leftover_change.clone ()
    }
}







pub struct AppendedTouchIterator<'a> {
    iterators : &'a mut [&'a mut dyn TouchIterator],

    accumulator : ChangeAccumulator,
    
    bell_iterator_index : usize,
    ruleoff_iterator_index : usize
}

impl AppendedTouchIterator<'_> {
    pub fn new<'a> (iterators : &'a mut [&'a mut dyn TouchIterator]) -> AppendedTouchIterator<'a> {
        assert! (iterators.len () > 0);

        let stage = iterators [0].stage ();
        for i in iterators.iter () {
            assert_eq! (stage, i.stage ());
        }

        AppendedTouchIterator {
            iterators : iterators,

            accumulator : ChangeAccumulator::new (stage),
            
            bell_iterator_index : 0,
            ruleoff_iterator_index : 0
        }
    }
}

impl<'a> TouchIterator for AppendedTouchIterator<'a> {
    fn next_bell (&mut self) -> Option<Bell> {
        loop {
            if self.bell_iterator_index >= self.iterators.len () {
                return None;
            }
        
            match self.iterators [self.bell_iterator_index].next_bell () {
                Some (x) => { 
                    return Some (self.accumulator.total ().slice () [x.as_usize ()]);
                }
                None => {
                    self.accumulator.accumulate (&self.iterators [self.bell_iterator_index].leftover_change ());

                    self.bell_iterator_index += 1;
                }
            }
        }
    }
    
    fn next_ruleoff (&mut self) -> Option<usize> {
        loop {
            if self.ruleoff_iterator_index >= self.iterators.len () {
                return None;
            }
        
            match self.iterators [self.ruleoff_iterator_index].next_ruleoff () {
                Some (x) => { 
                    return Some (x);
                }
                None => {
                    self.ruleoff_iterator_index += 1;
                }
            }
        }
    }

    fn reset (&mut self) {
        for i in 0..self.iterators.len () {
            self.iterators [i].reset ();
        }

        self.accumulator.reset ();

        self.bell_iterator_index = 0;
        self.ruleoff_iterator_index = 0;
    }

    fn length (&self) -> usize {
        let mut sum = 0;

        for i in self.iterators.iter () {
            sum += i.length ();
        }

        sum
    }

    fn stage (&self) -> Stage {
        self.iterators [0].stage ()
    }

    fn number_of_ruleoffs (&self) -> usize {
        let mut sum = 0;

        for i in self.iterators.iter () {
            sum += i.number_of_ruleoffs ();
        }

        sum
    }

    fn leftover_change (&self) -> Change {
        self.iterators [self.iterators.len () - 1].leftover_change ()
    }
}









pub struct ConcatTouchIterator<'a> {
    iterators : &'a mut [&'a mut dyn TouchIterator],
    
    bell_iterator_index : usize,
    ruleoff_iterator_index : usize
}

impl ConcatTouchIterator<'_> {
    pub fn new<'a> (iterators : &'a mut [&'a mut dyn TouchIterator]) -> ConcatTouchIterator<'a> {
        assert! (iterators.len () > 0);

        let stage = iterators [0].stage ();
        for i in iterators.iter () {
            assert_eq! (stage, i.stage ());
        }

        ConcatTouchIterator {
            iterators : iterators,
            
            bell_iterator_index : 0,
            ruleoff_iterator_index : 0
        }
    }
}

impl<'a> TouchIterator for ConcatTouchIterator<'a> {
    fn next_bell (&mut self) -> Option<Bell> {
        loop {
            if self.bell_iterator_index >= self.iterators.len () {
                return None;
            }
        
            match self.iterators [self.bell_iterator_index].next_bell () {
                Some (x) => { 
                    return Some (x);
                }
                None => {
                    self.bell_iterator_index += 1;
                }
            }
        }
    }
    
    fn next_ruleoff (&mut self) -> Option<usize> {
        loop {
            if self.ruleoff_iterator_index >= self.iterators.len () {
                return None;
            }
        
            match self.iterators [self.ruleoff_iterator_index].next_ruleoff () {
                Some (x) => { 
                    return Some (x);
                }
                None => {
                    self.ruleoff_iterator_index += 1;
                }
            }
        }
    }

    fn reset (&mut self) {
        for i in 0..self.iterators.len () {
            self.iterators [i].reset ();
        }

        self.bell_iterator_index = 0;
        self.ruleoff_iterator_index = 0;
    }

    fn length (&self) -> usize {
        let mut sum = 0;

        for i in self.iterators.iter () {
            sum += i.length ();
        }

        sum
    }

    fn stage (&self) -> Stage {
        self.iterators [0].stage ()
    }

    fn number_of_ruleoffs (&self) -> usize {
        let mut sum = 0;

        for i in self.iterators.iter () {
            sum += i.number_of_ruleoffs ();
        }

        sum
    }

    fn leftover_change (&self) -> Change {
        self.iterators [self.iterators.len () - 1].leftover_change ()
    }
}





pub struct TransfiguredTouchIterator<'a> {
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
    use crate::{ Touch, Transposition, PlaceNotation, Stage };
    
    #[test]
    fn basic_iterator () {
        for s_ref in &TOUCH_STRINGS {
            let s = *s_ref;
            let touch = Touch::from (s);

            assert_eq! (Touch::from_iterator (&mut touch.iterator ()), touch);
        }
    }

    #[test]
    fn row_iterator () {
        for s_ref in &TOUCH_STRINGS {
            let s = *s_ref;

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
    fn reflection () {
        for (pn, stage) in &[
            ("x58x16x12x36x12x58x14x18,12", Stage::MAJOR),
            ("36x7T.18x9T.50.36.14x1470.5T.16x9T.30.18x14.3T.50.14x1T,1T", Stage::MAXIMUS)
        ] {
            let pns = PlaceNotation::from_multiple_string (*pn, *stage);
            let reversed_pns : Vec<PlaceNotation> = pns.iter ().map (|x| x.reversed ()).collect ();

            assert_eq! (
                Touch::from (&pns [..]).reflected (), 
                Touch::from (&reversed_pns [..])
            );
        }
    }

    #[test]
    fn string_conversions () {
        for s_ref in &TOUCH_STRINGS {
            let s = *s_ref;
            let t = Touch::from (s);
            
            if t.to_string () != "" {
                assert_eq! (t.to_string () + "\n" + &t.leftover_change.to_string (), s);
            }
        }
    }
    
    const TOUCH_STRINGS : [&str; 4] = [
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
    ];
}
