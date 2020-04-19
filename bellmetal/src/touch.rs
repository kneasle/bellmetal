use crate::{
    Stage, Bell, Stroke,
    PlaceNotation,
    Change, ChangeAccumulator,
    Transposition,
    NaiveProver, ProvingContext, FullProvingContext,
    Method,
    TouchIterator
};

use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::Cloned;
use std::marker::PhantomData;
use crate::utils::AndNext;
use crate::proving::ProofGroups;

fn falseness_to_table (falseness_map : &Vec<Vec<usize>>) -> HashMap<usize, usize> {
    // Combine mappings
    let mut combination_tree : Vec<Option<usize>> = vec![None; falseness_map.len ()];

    for i in 1..falseness_map.len () {
        for j in 0..i {
            if falseness_map [i].len () == falseness_map [j].len () {
                let mut is_adjacent = true;
                let mut is_all_one = true;
                let mut is_all_minus_one = true;
                let mut is_first_diff_one = false;

                for k in 0..falseness_map [i].len () {
                    let diff = falseness_map [i] [k] as isize - falseness_map [j] [k] as isize;

                    is_all_one = is_all_one && diff == 1;
                    is_all_minus_one = is_all_minus_one && diff == -1;

                    if k == 0 && diff == 1 {
                        is_first_diff_one = true;
                    }

                    if diff != -1 && diff != 1 {
                        is_adjacent = false;
                        break;
                    }
                }

                if is_adjacent {
                    if is_all_minus_one {
                        assert_eq! (combination_tree [j], None);

                        combination_tree [j] = Some (i);
                    } else {
                        if !is_all_one && is_first_diff_one {
                            assert_eq! (combination_tree [i], None);

                            combination_tree [i] = Some (j);
                        } else {
                            assert_eq! (combination_tree [j], None);

                            combination_tree [j] = Some (i);
                        }
                    }
                }
            }
        }
    }

    // Map transiative closure / combine groups
    for i in 0..combination_tree.len () {
        let mut j = i;

        while let Some (x) = combination_tree [j] {
            if x == j {
                break;
            }

            j = x;
        }

        combination_tree [i] = Some (j);
    }

    let mut transiative_closure : Vec<usize> = combination_tree.iter ().map (|x| x.unwrap ()).collect ();

    // Renumber groups to lower, sequential numbers
    let mut group_renaming_map : HashMap<usize, usize> = HashMap::with_capacity (50);

    for i in 0..combination_tree.len () {
        let root = transiative_closure [i];

        if let Some (new_root) = group_renaming_map.get (&root) {
            transiative_closure [i] = *new_root;
        } else {
            transiative_closure [i] = group_renaming_map.len ();
            group_renaming_map.insert (root, group_renaming_map.len ());
        }
    }

    // Generate hash map from combined mappings
    let mut hash_map : HashMap<usize, usize> = HashMap::with_capacity (50);

    for (i, g) in falseness_map.iter ().enumerate () {
        for b in g {
            hash_map.insert (*b, transiative_closure [i]);
        }
    }

    hash_map
}


#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    pub index : usize,
    is_ruled_off : bool,
    call_char : char,
    method_name : Option<&'a str>,
    stroke : Stroke,
    bells : &'a [Bell]
}



static COLUMN_DELIMITER : &str = "  ";
static ANNOTATION_PADDING_LEFT : &str = "    ";
static ANNOTATION_PADDING_RIGHT : &str = "  ";
static FALSENESS_COLOURS : [&str; 14] = [
    "91;1", "92;1", "93;1", "94;1", "95;1", "96;1", "97;1",
    "31", "32", "33", "34", "35", "36", "37"
];

enum Position {
    Top,
    Middle,
    Bottom,
    Alone
}

fn get_position (table : &HashMap<usize, usize>, index : usize) -> Position {
    let g = table.get (&index).unwrap ();

    let above = if index == 0 { None } else { table.get (&(index - 1)) };
    let below = table.get (&(index + 1));

    if above == None || above != Some (g) {
        if below == None || below != Some (g) {
            Position::Alone
        } else {
            Position::Top
        }
    } else {
        if below == None || below != Some (g) {
            Position::Bottom
        } else {
            Position::Middle
        }
    }
}

impl Row<'_> {
    fn to_annotated_string (&self, table : &HashMap<usize, usize>) -> String {
        let mut s = String::with_capacity (self.stage ().as_usize () * 2);

        self.write_annotated_string (&mut s, table);

        s
    }

    fn write_annotated_string (&self, string : &mut String, table : &HashMap<usize, usize>) {
        match self.method_name {
            Some (s) => { string.push_str (s); }
            None => { string.push (' '); }
        }

        string.push (self.call_char);

        match table.get (&self.index) {
            Some (x) => {
                string.push_str ("\x1b[");
                string.push_str (FALSENESS_COLOURS [*x % FALSENESS_COLOURS.len ()]);
                string.push_str ("m");
                string.push (match get_position (table, self.index) {
                    Position::Alone  => { '[' },
                    Position::Top    => { '┏' },
                    Position::Middle => { '┃' },
                    Position::Bottom => { '┗' }
                });
                string.push_str ("\x1b[0m");
            }
            None => {
                string.push (' ');
            }
        }

        string.push (' ');

        self.write_pretty_string_with_stroke (string, self.stroke);

        string.push (' ');

        match table.get (&self.index) {
            Some (x) => {
                string.push_str ("\x1b[");
                string.push_str (FALSENESS_COLOURS [*x % FALSENESS_COLOURS.len ()]);
                string.push_str ("m");
                string.push (match get_position (table, self.index) {
                    Position::Alone  => { ']' },
                    Position::Top    => { '┓' },
                    Position::Middle => { '┃' },
                    Position::Bottom => { '┛' }
                });
                string.push_str ("\x1b[0m");
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





#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Touch {
    pub stage : Stage,
    pub length : usize,

    bells : Vec<Bell>,
    ruleoffs : Vec<usize>,
    method_names : HashMap<usize, String>,
    calls : HashMap<usize, char>,
    pub leftover_change : Change
}

impl Touch {
    pub fn row_iterator<'a> (&'a self) -> RowIterator<'a> {
        RowIterator::new (self)
    }

    pub fn iter<'a> (&'a self) -> BasicTouchIterator<'a> {
        BasicTouchIterator::new (self)
    }

    pub fn add_call (&mut self, index : usize, call_char : char) {
        self.calls.insert (index, call_char);
    }

    pub fn add_method_name (&mut self, index : usize, method_name : &str) {
        self.method_names.insert (index, String::from (method_name));
    }

    pub fn append_iterator<'a, 'b> (&'a mut self, iterator : &impl TouchIterator<'b>) {
        assert_eq! (self.stage, iterator.stage ());

        let len = self.length;

        self.bells.extend (iterator.bell_iter ());
        self.ruleoffs.extend (iterator.ruleoff_iter ().map (|x| x + len));
        self.calls.extend (iterator.call_iter ().map (|(ind, call)| (ind + len, call)));
        self.method_names.extend (iterator.method_name_iter ().map (|(ind, name)| (ind + len, name.clone ())));

        self.leftover_change.overwrite_from_iterator (&mut iterator.leftover_change_iter ());

        self.length += iterator.length ();
    }

    pub fn row_at (&self, index : usize) -> Row {
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
            method_name : match self.method_names.get (&index) {
                Some (s) => { Some (&s [..]) }
                None => { None }
            },
            stroke : Stroke::from_index (index),
            bells : self.slice_at (index)
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

    pub fn is_true_canonical (&self, canon : impl FnMut(&[Bell], &mut Change) -> ()) -> bool {
        NaiveProver { }.prove_touch_canonical (&self, canon)
    }

    pub fn is_true (&self) -> bool {
        NaiveProver { }.prove_touch (&self)
    }

    pub fn full_truth (&self) -> ProofGroups {
        NaiveProver { }.full_prove_touch (&self)
    }

    pub fn full_truth_canonical (&self, canon : impl FnMut(&[Bell], &mut Change) -> ()) -> ProofGroups {
        NaiveProver { }.full_prove_touch_canonical (&self, canon)
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
        let column_width = ANNOTATION_PADDING_LEFT.len () + stage + ANNOTATION_PADDING_RIGHT.len ();

        // Create useful strings
        let ruleoff_string = {
            let mut s = String::with_capacity (column_width);

            s.push_str (ANNOTATION_PADDING_LEFT);

            for _ in 0..stage {
                s.push ('─');
            }

            s.push_str (ANNOTATION_PADDING_RIGHT);

            s
        };

        let discontinuity_string = {
            let mut s = String::with_capacity (column_width);

            s.push_str (ANNOTATION_PADDING_LEFT);

            if stage <= 3 {
                s.push_str ("\x1b[31;1m");

                for _ in 0..stage {
                    s.push ('·');
                }

                s.push_str ("\x1b[0m");
            } else {
                let gap = if stage % 2 == 0 { stage / 2 - 2 } else { stage / 2 - 1 };

                for _ in 0..gap {
                    s.push (' ');
                }

                s.push_str ("\x1b[31;1m");

                for _ in 0..stage - 2 * gap {
                    s.push ('·');
                }

                s.push_str ("\x1b[0m");

                for _ in 0..gap {
                    s.push (' ');
                }
            }

            s.push_str (ANNOTATION_PADDING_RIGHT);

            s
        };

        let discontinuous_ruleoff_string = {
            let mut s = String::with_capacity (column_width);

            s.push_str (ANNOTATION_PADDING_LEFT);

            if stage <= 3 {
                s.push_str ("\x1b[31;1m");

                for _ in 0..stage {
                    s.push ('·');
                }

                s.push_str ("\x1b[0m");
            } else {
                let gap = if stage % 2 == 0 { stage / 2 - 2 } else { stage / 2 - 1 };

                for _ in 0..gap {
                    s.push ('─');
                }

                s.push_str ("\x1b[31;1m");

                for _ in 0..stage - 2 * gap {
                    s.push ('·');
                }

                s.push_str ("\x1b[0m");

                for _ in 0..gap {
                    s.push ('─');
                }
            }

            s.push_str (ANNOTATION_PADDING_RIGHT);

            s
        };

        let blank_string = {
            let mut s = String::with_capacity (column_width);

            for _ in 0..column_width {
                s.push (' ');
            }

            s
        };

        // Use the ruleoffs to decide how long each column should be
        let ideal_column_height = self.length / columns;
        let mut column_splits : Vec<usize> = Vec::with_capacity (columns);

        let mut next_ideal_split = ideal_column_height;
        let mut ruleoffs_used_this_split = 0;
        let mut last_r = 0;

        macro_rules! add {
            ($x : expr) => {
                if column_splits.len () == 0 || column_splits [column_splits.len () - 1] != $x {
                    column_splits.push ($x);
                }
            }
        }

        for &r in &self.ruleoffs {
            if r > next_ideal_split {
                if ruleoffs_used_this_split == 0 {
                    add! (r + 1);
                } else {
                    add! (last_r + 1);
                }

                next_ideal_split += ideal_column_height;
            }

            ruleoffs_used_this_split += 1;
            last_r = r;
        }

        // Initialise variables to generate the layout
        let mut fragments : HashMap<(usize, usize), String> = HashMap::with_capacity (self.length * 2 + columns);

        let mut x = 0;
        let mut y = 0;

        let mut height = 0;
        let mut width = 0;

        macro_rules! add {
            ($string : expr) => {
                fragments.insert ((x, y), $string);

                y += 1;

                if y > height {
                    height = y;
                }

                if x > width {
                    width = x;
                }
            }
        };

        for (i, (r, next_r)) in AndNext::new (self.row_iterator ()).enumerate () {
            // Determine if a discontinuity has happened
            // Start new column if required, and add the row to the bottom of the last column
            let needs_new_column = match column_splits.get (x) {
                None => false,
                Some (&v) => i == v
            };

            let is_continuous = match next_r {
                None => true,
                Some (x) => r.is_continuous_with (x)
            };

            if needs_new_column {
                add! (r.to_annotated_string (&truth_table));

                x += 1;
                y = 0;
            }

            add! (r.to_annotated_string (&truth_table));

            // Push the ruleoffs and discontinuity strings
            match (r.is_ruled_off, is_continuous) {
                (false, false) => {
                    add! (discontinuity_string.clone ());
                }
                (true, false) => {
                    add! (discontinuous_ruleoff_string.clone ());
                }
                (true, true) => {
                    add! (ruleoff_string.clone ());
                }
                _ => { }
            }
        }

        add! ({
            let mut s = String::with_capacity (100);

            s.push_str (ANNOTATION_PADDING_LEFT);

            self.leftover_change.write_pretty_string (&mut s);

            s
        });

        let mut final_string = (0..height).map (
            |y| (0..=width).map (
                |x| fragments.get (&(x, y)).unwrap_or (&blank_string)
            ).join (COLUMN_DELIMITER)
        ).join ("\n");

        final_string.push ('\n');
        final_string.push_str (&self.coloured_summary_string ());

        final_string
    }

    pub fn pretty_string (&self, truth : Option<&Vec<Vec<usize>>>) -> String {
        self.pretty_string_multi_column (1, truth)
    }

    // Tested in touch_generation.rs
    pub fn coloured_summary_string (&self) -> String {
        let (f, b) = self.number_of_4_bell_runs ();

        format! (
            "\x1b[94m{}\x1b[0m changes, {}.  Score: \x1b[93m{}\x1b[0m. {} 4-bell runs ({}f, {}b)",
            &self.length.to_string (),
            if self.is_true () { "\x1b[92mtrue\x1b[0m" } else { "\x1b[91mfalse\x1b[0m" },
            self.music_score (),
            f + b,
            f, b
        )
    }

    pub fn summary_string (&self) -> String {
        let (f, b) = self.number_of_4_bell_runs ();

        format! (
            "{} changes, {}.  Score: {}. {} 4-bell runs ({}f, {}b)",
            &self.length.to_string (),
            if self.is_true () { "true" } else { "false" },
            self.music_score (),
            f + b,
            f, b
        )
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
            for b in accumulator.total ().iter () {
                self.bells.push (b);
            }

            accumulator.accumulate_iterator (p.iter ());
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

    pub fn inverted (&self) -> Touch {
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
            method_names : self.method_names.clone (),
            leftover_change : self.leftover_change.inverted ()
        }
    }
}

impl Touch {
    pub fn empty (stage : Stage) -> Touch {
        Touch {
            stage : stage,
            length : 0usize,

            bells : Vec::with_capacity (0),
            ruleoffs : Vec::with_capacity (0),
            calls : HashMap::with_capacity (0),
            method_names : HashMap::with_capacity (0),
            leftover_change : Change::empty ()
        }
    }

    pub fn from_changes (changes : &[Change], leftover_change : Change) -> Touch {
        let mut bells : Vec<Bell> = Vec::with_capacity (changes.len () * leftover_change.stage ().as_usize ());

        for c in changes {
            bells.extend (c.iter ());
        }

        Touch {
            stage : leftover_change.stage (),
            length : changes.len (),

            bells : bells,
            ruleoffs : vec![changes.len () - 1],
            calls : HashMap::with_capacity (0),
            method_names : HashMap::with_capacity (0),
            leftover_change : leftover_change
        }
    }

    pub fn with_capacity (
        stage : Stage,
        change_capacity : usize,
        ruleoff_capacity : usize,
        call_capacity : usize,
        method_name_capacity : usize,
    ) -> Touch {
        Touch {
            stage : stage,
            length : 0usize,

            bells : Vec::with_capacity (change_capacity * stage.as_usize ()),
            ruleoffs : Vec::with_capacity (ruleoff_capacity),
            calls : HashMap::with_capacity (call_capacity),
            method_names : HashMap::with_capacity (method_name_capacity),
            leftover_change : Change::rounds (stage)
        }
    }

    pub fn single_course (method : &Method, course_head : &Change) -> Touch {
        let mut accumulator = ChangeAccumulator::new (method.stage);
        let mut touch = Touch::empty (method.stage);

        accumulator.set (course_head);
        touch.stage = method.stage;

        loop {
            touch.append_iterator (&method.plain_lead.iter ().transfigure (accumulator.total ()));
            accumulator.accumulate (method.lead_head ());

            if accumulator.total () == course_head {
                break;
            }
        }

        touch
    }

    pub fn from_iterator<'b, I> (iterator : &I) -> Touch where I : TouchIterator<'b>, I : Sized {
        let mut touch = Touch::empty (iterator.stage ());

        touch.append_iterator (iterator);

        touch
    }
}

impl From<&[PlaceNotation]> for Touch {
    fn from (place_notations : &[PlaceNotation]) -> Touch {
        let mut touch = Touch::empty (Stage::ZERO);

        touch.overwrite_from_place_notations (place_notations);

        touch.ruleoffs.push (place_notations.len () - 1);

        touch
    }
}

impl From<&str> for Touch {
    fn from (string : &str) -> Touch {
        let mut touch = Touch::empty (Stage::ZERO);

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




// An iterator that dereferences the values coming from an iterator without using map
pub struct CallDerefIter<'a, T : Iterator<Item = (&'a usize, &'a char)>> {
    iterator : T,
    phantom : PhantomData<&'a ()>
}

impl<'a, T : Iterator<Item = (&'a usize, &'a char)>> Iterator for CallDerefIter<'a, T> {
    type Item = (usize, char);

    fn next (&mut self) -> Option<(usize, char)> {
        match self.iterator.next () {
            None => {
                None
            }
            Some ((ind, call)) => {
                Some ((*ind, *call))
            }
        }
    }

    fn size_hint (&self) -> (usize, Option<usize>) {
        self.iterator.size_hint ()
    }
}

// An iterator that dereferences the values coming from an iterator without using map
pub struct MethodNameDerefIter<'a, T : Iterator<Item = (&'a usize, &'a String)>> {
    iterator : T,
    phantom : PhantomData<&'a ()>
}

impl<'a, T : Iterator<Item = (&'a usize, &'a String)>> Iterator for MethodNameDerefIter<'a, T> {
    type Item = (usize, &'a String);

    fn next (&mut self) -> Option<(usize, &'a String)> {
        match self.iterator.next () {
            None => {
                None
            }
            Some ((ind, name)) => {
                Some ((*ind, name))
            }
        }
    }

    fn size_hint (&self) -> (usize, Option<usize>) {
        self.iterator.size_hint ()
    }
}

// The full iterator
pub struct BasicTouchIterator<'a> {
    touch : &'a Touch,
}

impl BasicTouchIterator<'_> {
    pub fn new<'a> (touch : &'a Touch) -> BasicTouchIterator<'a> {
        BasicTouchIterator {
            touch : touch,
        }
    }
}

impl<'a> TouchIterator<'a> for BasicTouchIterator<'a> {
    type BellIter = Cloned<std::slice::Iter<'a, Bell>>;
    type RuleoffIter = Cloned<std::slice::Iter<'a, usize>>;
    type CallIter = CallDerefIter<'a, std::collections::hash_map::Iter<'a, usize, char>>;
    type MethodNameIter = MethodNameDerefIter<'a, std::collections::hash_map::Iter<'a, usize, String>>;
    type LeftoverChangeIter = std::iter::Cloned<std::slice::Iter<'a, Bell>>;

    fn bell_iter (&self) -> Self::BellIter {
        self.touch.bells.iter ().cloned ()
    }

    fn ruleoff_iter (&self) -> Self::RuleoffIter {
        self.touch.ruleoffs.iter ().cloned ()
    }

    fn call_iter (&self) -> Self::CallIter {
        CallDerefIter {
            iterator : self.touch.calls.iter (),
            phantom : PhantomData
        }
    }

    fn method_name_iter (&self) -> Self::MethodNameIter {
        MethodNameDerefIter {
            iterator : self.touch.method_names.iter (),
            phantom : PhantomData
        }
    }

    fn length (&self) -> usize {
        self.touch.length
    }

    fn stage (&self) -> Stage {
        self.touch.stage
    }

    fn leftover_change_iter (&self) -> Self::LeftoverChangeIter {
        self.touch.leftover_change.iter ()
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

            assert_eq! (Touch::from_iterator (&mut touch.iter ()), touch);
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
            for b in touch.leftover_change.iter () {
                match chars.next () {
                    Some (c) => { assert_eq! (b.as_char (), c); }
                    None => { panic! ("Touch yielded too many bells"); }
                }
            }

            assert_eq! (chars.next (), None);
        }
    }

    #[test]
    fn inversion () {
        for (pn, stage) in &[
            ("x58x16x12x36x12x58x14x18,12", Stage::MAJOR),
            ("36x7T.18x9T.50.36.14x1470.5T.16x9T.30.18x14.3T.50.14x1T,1T", Stage::MAXIMUS)
        ] {
            let pns = PlaceNotation::from_multiple_string (*pn, *stage);
            let reversed_pns : Vec<PlaceNotation> = pns.iter ().map (|x| x.reversed ()).collect ();

            assert_eq! (
                Touch::from (&pns [..]).inverted (),
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
