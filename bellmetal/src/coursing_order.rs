use crate::{Bell, Change, Place, Stage, Stroke, Transposition};

use std::cmp;
use std::cmp::Ordering;
use std::fmt;
use std::ops::Index;

/// Records a section of a [CoursingOrder] where the [Bell]s would form a run in Plain Bob (and
/// many other common methods.
/// For example, the coursing order `975346280` has two run segments (the `75346` generating
/// backstroke 76543s, and the wrapped `80 97` generating backstroke 7890s).  Note that the 7th is
/// contained in both run segements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSection {
    /// The index of the left-most [Bell] that is part of this `RunSection`
    pub start: isize,
    /// The index of the [Bell] that is at the centre of the run (and will appear at the back of
    /// the runs).
    pub centre: isize,
    /// The index of the right-most [Bell] that is part of this `RunSection`
    pub end: isize,
    /// The [Bell] that starts the run.  This will be either at [start](RunSection::start) or
    /// [end](RunSection::end) of the run segment.
    pub bell_start: Bell,
    /// The [Bell] that ends the run.  This will be at index [RunSection::centre] of the
    /// [CoursingOrder] that this is a section of.
    pub bell_end: Bell,
    /// The [Stroke] of the run
    pub stroke: Stroke,
}

impl Ord for RunSection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bell_end.cmp(&other.bell_end)
    }
}

impl PartialOrd for RunSection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A struct to store the representation of a coursing order.  Since coursing orders repeat
/// forever, this implementation stores the repeating subsequence of this infinite sequence which
/// starts with the heaviest bell in the order.
/// This implementation also does not assume Plain Bob lead heads, although many of the methods
/// (such as [from_leadhead](CoursingOrder::from_leadhead),
/// [to_coursehead](CoursingOrder::to_coursehead)) do assume this for simplicity.
#[derive(PartialEq, Eq, Hash)]
pub struct CoursingOrder {
    order: Vec<Bell>, // order will always start with the heaviest bell in the coursing order
}

impl CoursingOrder {
    /// Creates an empty `CoursingOrder`, i.e. one that contains no bells.  Like
    /// [`Vec::with_capacity(0)`](Vec::with_capacity), this will not allocate memory.
    ///
    /// # Example
    /// ```
    /// use bellmetal::CoursingOrder;
    ///
    /// let empty_coursing_order = CoursingOrder::empty();
    ///
    /// assert_eq!(empty_coursing_order.to_string(), "");
    /// ```
    pub fn empty() -> CoursingOrder {
        CoursingOrder {
            order: Vec::with_capacity(0),
        }
    }

    /// Creates a coursing order from a [slice](std::slice) of [Bell]s.  This slice must start with
    /// the heaviest bell in the order.
    fn from_slice(slice: &[Bell]) -> CoursingOrder {
        CoursingOrder {
            order: slice.iter().copied().collect(),
        }
    }

    /// Creates a coursing order from a [Transposition] representing a lead head.  This assumes
    /// that the method who's coursing order this is has Plain Bob lead heads, but nearly all
    /// methods do.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Change, CoursingOrder, Stage};
    ///
    /// assert_eq!(
    ///     CoursingOrder::from_leadhead(&Change::rounds(Stage::MINOR)).to_string(),
    ///     "65324"
    /// );
    /// ```
    pub fn from_leadhead(lh: &impl Transposition) -> CoursingOrder {
        let mut co = CoursingOrder::empty();

        co.overwrite_from_leadhead(lh);

        co
    }

    /// Converts a [CoursingOrderIterator] into a `CoursingOrder`, regardless of how much of the
    /// iterator has already been consumed.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{PlainCoursingOrderIterator, CoursingOrderIterator, CoursingOrder, Stage};
    ///
    /// let mut plain_course_iterator = PlainCoursingOrderIterator::new(Stage::MAJOR);
    ///
    /// // Consume some of the iterator
    /// for i in 0..10 {
    ///     plain_course_iterator.next();
    /// }
    ///
    /// let coursing_order = CoursingOrder::from_iterator(&mut plain_course_iterator);
    ///
    /// assert_eq!(coursing_order.to_string(), "8753246");
    /// ```
    pub fn from_iterator(iterator: &mut impl CoursingOrderIterator) -> CoursingOrder {
        let mut co = CoursingOrder::empty();

        co.overwrite_from_iterator(iterator);

        co
    }

    /// Overwrites an existing `CoursingOrder` from a string slice representing a coursing order.
    /// The length of the existing `CoursingOrder` does not necessarily need to match the length of
    /// the `CoursingOrder` after overwriting.  In order to produce a valid `CoursingOrder`, the
    /// heaviest bell in the string must appear first.
    ///
    /// # Panics
    /// Panics if the string contains characters that aren't valid bell names.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{CoursingOrder};
    ///
    /// let mut coursing_order = CoursingOrder::empty();
    ///
    /// coursing_order.overwrite_from_string("8764235");
    ///
    /// assert_eq!(coursing_order.to_string(), "8764235");
    /// ```
    pub fn overwrite_from_string(&mut self, string: &str) {
        self.order.clear();
        self.order.reserve(string.len());

        for c in string.chars() {
            self.order.push(Bell::from(c));
        }
    }

    /// Overwrites an existing `CoursingOrder` with the contents of a [CoursingOrderIterator].
    /// This will automatically ensure that the heaviest bell is at the front of the coursing
    /// order.  This is used (along with [CoursingOrder::empty]) in [CoursingOrder::from_iterator]
    /// to populate a blank `CoursingOrder` with the contents of an iterator.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Change, CoursingOrder, Stage};
    ///
    /// let mut plain_course = CoursingOrder::from_leadhead(&Change::rounds(Stage::MAJOR));
    /// let mut mega_tittums = CoursingOrder::from("8765432");
    ///
    /// plain_course.overwrite_from_iterator(&mut mega_tittums.iter());
    ///
    /// assert_eq!(plain_course.to_string(), "8765432");
    /// ```
    pub fn overwrite_from_iterator(&mut self, iterator: &mut impl CoursingOrderIterator) {
        let heaviest_bell = iterator.seek_heaviest_bell();

        // Copy the iterator into the vector
        let len = iterator.length();

        self.order.clear();
        self.order.reserve(len);

        self.order.push(heaviest_bell);
        for _ in 0..len - 1 {
            let x = iterator.next();

            self.order.push(x);
        }
    }

    /// Overwrite a `CoursingOrder` so that it represents the course that contains by a given lead
    /// head.  This can change the length of the `CoursingOrder`.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Change, CoursingOrder};
    ///
    /// let mut coursing_order = CoursingOrder::empty();
    ///
    /// coursing_order.overwrite_from_leadhead(&Change::from("134256"));
    ///
    /// assert_eq!(coursing_order.to_string(), "65432");
    /// ```
    pub fn overwrite_from_leadhead(&mut self, lead_head: &impl Transposition) {
        self.overwrite_from_iterator(&mut LeadheadCoursingOrderIterator::new(lead_head));
    }

    /// Tests a run segment that will represent an ascending run.  Used in
    /// [CoursingOrder::get_run_sections].
    fn test_run_segment_up(&self, root: isize, side: isize, vec: &mut Vec<RunSection>) {
        let mut len = 0;
        let mut current_index = 0;
        let mut last_index = 0;

        let a = self[root].as_isize();

        // Zig zag out from the centre until a bell is found that doesn't follow the run.
        for (l, index) in ZigZagIterator::new(root, side).enumerate() {
            if self[index].as_isize() != a + l as isize {
                len = l;

                break;
            }

            last_index = current_index;
            current_index = index;
        }

        // If the run is longer than 4 bells, then collect all this information into a RunSection
        // struct and add it to the vector
        if len >= 4 {
            vec.push(RunSection {
                start: cmp::min(current_index, last_index),
                centre: root,
                end: cmp::max(current_index, last_index),
                bell_start: self[current_index],
                bell_end: self[root],
                stroke: if root < side {
                    Stroke::Back
                } else {
                    Stroke::Hand
                },
            });
        }
    }

    /// Tests a run segment that will represent a descending run.  Used in
    /// [CoursingOrder::get_run_sections].
    fn test_run_segment_down(&self, root: isize, side: isize, vec: &mut Vec<RunSection>) {
        let mut len = 0;
        let mut current_index = 0;
        let mut last_index = 0;

        let a = self[root].as_isize();

        // Zig zag out from the centre until a bell is found that doesn't follow the run.
        for (l, index) in ZigZagIterator::new(root, side).enumerate() {
            if self[index].as_isize() != a - l as isize {
                len = l;

                break;
            }

            last_index = current_index;
            current_index = index;
        }

        // If the run is longer than 4 bells, then collect all this information into a RunSection
        // struct and add it to the vector
        if len >= 4 {
            vec.push(RunSection {
                start: cmp::min(current_index, last_index),
                centre: root,
                end: cmp::max(current_index, last_index),
                bell_start: self[current_index],
                bell_end: self[root],
                stroke: if root < side {
                    Stroke::Back
                } else {
                    Stroke::Hand
                },
            });
        }
    }

    /// Generate all the [RunSection]s contained in this coursing order that would generate runs of
    /// 4 bells or longer in Plain Bob (and all other methods which follow Plain Bob's music
    /// style).  See [RunSection] for more details.  For example, the coursing order `097534628`
    /// contains two run sections: the `75346` part which generates `76543`s at backstroke and the
    /// wrapped `8 097` part that generates `7890`s at handstroke too.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Bell, CoursingOrder, RunSection, Stroke};
    ///
    /// let coursing_order = CoursingOrder::from("097534628");
    ///
    /// assert_eq!(
    ///     coursing_order.get_run_sections(),
    ///     vec![
    ///         // The 76543s section
    ///         RunSection {
    ///             // Location of the run segment within the coursing order
    ///             start: 2,
    ///             centre: 4,
    ///             end: 6,
    ///
    ///             // Info about what music the run corresponds to
    ///             bell_start: Bell::from('7'),
    ///             bell_end: Bell::from('3'),
    ///             stroke: Stroke::Back
    ///         },
    ///         // The 7890s section
    ///         RunSection {
    ///             // Location of the run segment within the coursing order
    ///             // (note how it starts at index -1 showing how it wraps over the tenor)
    ///             start: -1,
    ///             centre: 0,
    ///             end: 2,
    ///
    ///             // Info about what music the run corresponds to
    ///             bell_start: Bell::from('7'),
    ///             bell_end: Bell::from('0'),
    ///             stroke: Stroke::Back
    ///         }
    ///     ]
    /// );
    /// ```
    pub fn get_run_sections(&self) -> Vec<RunSection> {
        let mut run_sections: Vec<RunSection> = Vec::with_capacity(10);

        // run_sections[0].

        for i in 0..self.order.len() as isize {
            let a = self[i - 1].as_isize();
            let b = self[i].as_isize();

            if b - a == 1 {
                self.test_run_segment_up(i - 1, i, &mut run_sections);
                self.test_run_segment_down(i, i - 1, &mut run_sections);
            }
            if a - b == 1 {
                self.test_run_segment_up(i, i - 1, &mut run_sections);
                self.test_run_segment_down(i - 1, i, &mut run_sections);
            }
        }

        run_sections.sort();

        run_sections
    }

    /// Writes all the [Bell]s in this [CoursingOrder] into a mutable [String], with no formatting.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Change, CoursingOrder};
    ///
    /// let mut s = String::with_capacity(10);
    ///
    /// CoursingOrder::from("8753462").into_string(&mut s);
    ///
    /// assert_eq!(s, "8753462");
    /// ```
    pub fn into_string(&self, string: &mut String) {
        string.reserve(self.order.len());

        for b in &self.order {
            string.push(b.as_char());
        }
    }

    /// Writes all the [Bell]s in this [CoursingOrder] to a [String], with no formatting.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Change, CoursingOrder};
    ///
    /// assert_eq!(
    ///     CoursingOrder::from("8753462").to_string(),
    ///     "8753462"
    /// );
    /// ```
    pub fn to_string(&self) -> String {
        let mut s = String::with_capacity(0);

        self.into_string(&mut s);

        s
    }

    /// Returns a [CoursingOrderIterator] that will iterate the bells in this `CoursingOrder`.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Bell, Change, CoursingOrder, CoursingOrderIterator, Stage};
    ///
    /// let coursing_order = CoursingOrder::from_leadhead(&Change::rounds(Stage::MAJOR));
    ///
    /// let mut coursing_order_iter = coursing_order.iter();
    ///
    /// assert_eq!(coursing_order_iter.next(), Bell::from('8'));
    /// assert_eq!(coursing_order_iter.next(), Bell::from('7'));
    /// assert_eq!(coursing_order_iter.next(), Bell::from('5'));
    /// assert_eq!(coursing_order_iter.next(), Bell::from('3'));
    /// assert_eq!(coursing_order_iter.next(), Bell::from('2'));
    /// // ....
    ///
    /// assert_eq!(coursing_order_iter.length(), 7);
    /// ```
    pub fn iter(&self) -> BasicCoursingOrderIterator {
        BasicCoursingOrderIterator::new(self)
    }

    /// Returns a [Change] representing the course head of this course, assuming Plain Bob lead end
    /// methods.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Change, CoursingOrder};
    ///
    /// let coursing_order = CoursingOrder::from("097356428");
    ///
    /// assert_eq!(coursing_order.to_coursehead(), Change::from("1654327890"));
    /// ```
    pub fn to_coursehead(&self) -> Change {
        BasicCoursingOrderIterator::new(self).to_coursehead()
    }

    /// Returns a [String] with a summary of this `CoursingOrder`, containing:
    /// - the [RunSection]s contained in this course and what stroke those runs are on.
    /// - 87s or 09s or TEs at backstroke.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{CoursingOrder};
    ///
    /// assert_eq!(
    ///     CoursingOrder::from("097356428").canonical_string(),
    ///     "CO: <97356428> runs: 23456s(H) 7890s(B)"
    /// );
    ///
    /// assert_eq!(
    ///     CoursingOrder::from("8542367").canonical_string(),
    ///     "CO: <542367> no runs. 87s"
    /// );
    /// ```
    pub fn canonical_string(&self) -> String {
        let mut string = String::with_capacity(100);

        // Write out the coursing order as a string of bells
        string.push_str("CO: <");

        for b in self.order.iter().skip(1) {
            string.push(b.as_char());
        }

        string.push_str("> ");

        let run_sections = self.get_run_sections();

        // Add "no runs." or "runs:" depending on if this will be followed by some run segments
        if run_sections.len() == 0 {
            string.push_str("no runs.");
        } else {
            string.push_str("runs:");
        }

        // Add the run sections to the string
        for r in run_sections {
            let a = r.bell_start.as_usize();
            let b = r.bell_end.as_usize();

            string.push_str(" ");

            if b > a {
                for i in a..=b {
                    string.push(Bell::from(i).as_char());
                }
            } else {
                for i in (b..=a).rev() {
                    string.push(Bell::from(i).as_char());
                }
            }

            string.push('s');

            string.push_str(if r.stroke == Stroke::Hand {
                "(H)"
            } else {
                "(B)"
            });
        }

        // Detect if the tenors are hunting the 'wrong' way round, and will generate e.g. 87s
        let tenor_number = self[0].as_usize();

        if self[self.order.len() as isize - 1].as_usize() == tenor_number - 1 {
            string.push(' ');
            string.push(Bell::from(tenor_number).as_char());
            string.push(Bell::from(tenor_number - 1).as_char());
            string.push('s');
        }

        string
    }
}

impl Index<isize> for CoursingOrder {
    type Output = Bell;

    fn index(&self, x: isize) -> &Bell {
        let l = self.order.len() as isize;

        &self.order[(((x % l) + l) % l) as usize]
    }
}

impl From<&str> for CoursingOrder {
    fn from(s: &str) -> CoursingOrder {
        let mut coursing_order = CoursingOrder::empty();

        coursing_order.overwrite_from_string(s);

        coursing_order
    }
}

impl fmt::Debug for CoursingOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.to_string())
    }
}

/// Combine two [CoursingOrderIterator]s together (one generating the [Place]s and the other
/// generating the [Bell]s) into one lead head, filling all the other [Place]s with the bells from
/// rounds.  Used internally to generate the lead heads from [CoursingOrder]s.
fn merge_iterators_to_lead_head<T: CoursingOrderIterator, S: CoursingOrderIterator>(
    bell_iter: &mut T,
    place_iter: &mut S,
    stage: Stage,
) -> Change {
    let mut vec: Vec<Bell> = vec![Bell::from(0); stage.as_usize()];

    for _ in 0..bell_iter.length() {
        vec[place_iter.next().as_usize()] = bell_iter.next();
    }

    Change::new(vec)
}

/// Returns a [Change] the first lead head of plain bob on a given [Stage].
///
/// # Example
/// ```
/// use bellmetal::{first_plain_bob_lead_head, Change, Stage};
///
/// assert_eq!(first_plain_bob_lead_head(Stage::DOUBLES), Change::from("13524"));
/// assert_eq!(first_plain_bob_lead_head(Stage::MINOR), Change::from("135264"));
/// assert_eq!(first_plain_bob_lead_head(Stage::ROYAL), Change::from("1352749608"));
/// ```
pub fn first_plain_bob_lead_head(stage: Stage) -> Change {
    let mut bell_iterator = PlainCoursingOrderIterator::new(stage);
    let mut place_iterator = PlainCoursingOrderIterator::new(stage);

    place_iterator.seek(Bell::from(1)); // Seek to 2nds place
    bell_iterator.seek(Bell::from(2)); // Seek to the 3

    merge_iterators_to_lead_head(&mut bell_iterator, &mut place_iterator, stage)
}

/// Returns an arbitrary Plain Bob lead head, given its [Stage] and how many leads of Plain Bob it
/// corresponds to.
///
/// # Example
/// ```
/// use bellmetal::{plain_bob_lead_head, Change, Stage};
///
/// // f-group Minor method
/// assert_eq!(plain_bob_lead_head(Stage::MINOR, -1), Change::from("142635"));
///
/// // c-group Major method
/// assert_eq!(plain_bob_lead_head(Stage::MAJOR, 3), Change::from("17856342"));
/// ```
pub fn plain_bob_lead_head(stage: Stage, power: isize) -> Change {
    first_plain_bob_lead_head(stage).pow(power)
}

struct ZigZagIterator {
    current_value: isize,
    next_value: isize,
}

impl ZigZagIterator {
    pub fn new(current_value: isize, next_value: isize) -> ZigZagIterator {
        ZigZagIterator {
            current_value,
            next_value,
        }
    }
}

impl Iterator for ZigZagIterator {
    type Item = isize;

    fn next(&mut self) -> Option<isize> {
        let current_value = self.current_value;

        self.current_value = self.next_value;
        if self.next_value > current_value {
            self.next_value = current_value - 1;
        } else {
            self.next_value = current_value + 1;
        }

        Some(current_value)
    }
}

/// A trait similar to [Iterator], with a few differences specific to representing coursing orders:
/// - The [next](CoursingOrderIterator::next) method returns a [Bell] rather than a `Option<Bell>`
/// since coursing orders must repeat forever.
/// - The [length](CoursingOrderIterator::length) method should be a cheap way to determine the
/// length of one repeating section of the coursing order.
pub trait CoursingOrderIterator {
    /// Returns the next [Bell] in the coursing order.
    fn next(&mut self) -> Bell;

    /// Returns the length of one repeating section of the coursing order.
    fn length(&self) -> usize;

    /// Collects this `CoursingOrderIterator` into a [CoursingOrder].
    ///
    /// # Example
    /// ```
    /// use bellmetal::{CoursingOrderIterator, CoursingOrder, PlainCoursingOrderIterator, Stage};
    ///
    /// assert_eq!(
    ///     PlainCoursingOrderIterator::new(Stage::MAJOR).collect(),
    ///     CoursingOrder::from("8753246")
    /// );
    /// ```
    fn collect(&mut self) -> CoursingOrder
    where
        Self: Sized,
    {
        CoursingOrder::from_iterator(self)
    }

    /// Move through the iterator until it consumes a given [Bell].  This implementation will hang
    /// forever if the [Bell] is not in the [CoursingOrderIterator] - wheras
    /// [seek_safe](CoursingOrderIterator::seek_safe) will not.  After
    /// [seek](CoursingOrderIterator::seek) is run, the next [Bell] in the [CoursingOrderIterator]
    /// will be the one **after** the [Bell] seeked.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{
    ///     Bell, CoursingOrderIterator, CoursingOrder, Stage
    /// };
    ///
    /// let coursing_order = CoursingOrder::from("8724653");
    /// let mut iter = coursing_order.iter();
    ///
    /// iter.seek(Bell::from('4'));
    ///
    /// assert_eq!(iter.next(), Bell::from('6'));
    /// assert_eq!(iter.next(), Bell::from('5'));
    /// assert_eq!(iter.next(), Bell::from('3'));
    /// ```
    fn seek(&mut self, bell: Bell) {
        // Seek to the required bell with a horrendous loop with side effects in the guard
        while self.next() != bell {}
    }

    /// Move through the iterator until it consumes a given [Bell], panicking if this would cause
    /// an infinte loop.  After [seek](CoursingOrderIterator::seek) is run, the next [Bell] in the
    /// [CoursingOrderIterator] will be the one **after** the [Bell] seeked.
    ///
    /// # Example
    /// ```
    /// use bellmetal::{
    ///     Bell, CoursingOrderIterator, CoursingOrder, Stage
    /// };
    ///
    /// let coursing_order = CoursingOrder::from("8724653");
    /// let mut iter = coursing_order.iter();
    ///
    /// iter.seek_safe(Bell::from('4'));
    ///
    /// assert_eq!(iter.next(), Bell::from('6'));
    /// assert_eq!(iter.next(), Bell::from('5'));
    /// assert_eq!(iter.next(), Bell::from('3'));
    /// ```
    fn seek_safe(&mut self, bell: Bell) {
        // Keep track of the first bell seen, so that if we see it again we can panic
        let start_bell = self.next();
        let mut next_bell = start_bell;

        while next_bell != bell {
            next_bell = self.next();

            // Panic if we see the first again
            if next_bell == start_bell {
                panic!("Bell not found in coursing order");
            }
        }
    }

    /// Seeks the heaviest (highest numbered) [Bell] in the `CoursingOrderIterator` and returns
    /// that [Bell], leaving this `CoursingOrderIterator` such that the next [Bell] will be the
    /// [Bell] after the heaviest [Bell].
    ///
    /// # Example
    /// ```
    /// use bellmetal::{
    ///     Bell, CoursingOrderIterator, CoursingOrder, Stage
    /// };
    ///
    /// let coursing_order = CoursingOrder::from("8724653");
    /// let mut iter = coursing_order.iter();
    ///
    /// // Iterate a few times, and assert that the next bell is the 2
    /// iter.next();
    /// iter.next();
    ///
    /// assert_eq!(iter.next(), Bell::from('2'));
    ///
    /// // Seek to the heaviest bell and assert that it is in fact the 8
    /// assert_eq!(iter.seek_heaviest_bell(), Bell::from('8'));
    ///
    /// // We've moved through the iterator, and now the next bell is the 7
    /// assert_eq!(iter.next(), Bell::from('7'));
    /// ```
    fn seek_heaviest_bell(&mut self) -> Bell {
        let heaviest_bell = {
            let mut heaviest_bell = 0;

            // Find what the heaviest bell is
            for _ in 0..self.length() {
                heaviest_bell = std::cmp::max(self.next().as_usize(), heaviest_bell);
            }

            Bell::from(heaviest_bell)
        };

        self.seek(heaviest_bell);

        heaviest_bell
    }

    /// Converts this `CoursingOrderIterator` into a [Change] representing the course head of this
    /// course, assuming Plain Bob lead ends.  The 'course head' is in this case defined to be the
    /// lead head where the heaviest working [Bell] is at the back of the [Change].  Used by
    /// [CoursingOrder::to_coursehead].
    ///
    /// # Example
    /// ```
    /// use bellmetal::{Change, CoursingOrder, CoursingOrderIterator, Stage};
    ///
    /// let coursing_order = CoursingOrder::from("8357642");
    /// let mut iter = coursing_order.iter();
    ///
    /// assert_eq!(iter.to_coursehead(), Change::from("16745238"));
    /// ```
    fn to_coursehead(&mut self) -> Change
    where
        Self: Sized,
    {
        let stage = self.length() + 1;

        let mut iter = PlainCoursingOrderIterator::new(Stage::from(stage));

        iter.seek_heaviest_bell();
        self.seek_heaviest_bell();

        merge_iterators_to_lead_head(self, &mut iter, Stage::from(stage))
    }
}

pub struct BasicCoursingOrderIterator<'a> {
    coursing_order: &'a CoursingOrder,
    index: usize,
}

impl BasicCoursingOrderIterator<'_> {
    pub fn new<'a>(coursing_order: &'a CoursingOrder) -> BasicCoursingOrderIterator<'a> {
        BasicCoursingOrderIterator {
            coursing_order,
            index: 0,
        }
    }
}

impl<'a> CoursingOrderIterator for BasicCoursingOrderIterator<'a> {
    fn next(&mut self) -> Bell {
        let b = self.coursing_order[self.index as isize];

        self.index += 1;

        b
    }

    fn length(&self) -> usize {
        self.coursing_order.order.len()
    }
}

pub struct LeadheadCoursingOrderIterator<'a, T: Transposition> {
    leadhead: &'a T,
    iterator: PlainCoursingOrderIterator,
}

impl<T: Transposition> LeadheadCoursingOrderIterator<'_, T> {
    pub fn new<'a>(leadhead: &'a T) -> LeadheadCoursingOrderIterator<'a, T> {
        LeadheadCoursingOrderIterator {
            leadhead,
            iterator: PlainCoursingOrderIterator::new(leadhead.stage()),
        }
    }
}

impl<'a, T: Transposition> CoursingOrderIterator for LeadheadCoursingOrderIterator<'a, T> {
    fn next(&mut self) -> Bell {
        self.leadhead
            .bell_at(Place::from(self.iterator.next().as_usize()))
    }

    fn length(&self) -> usize {
        self.iterator.length()
    }
}

pub struct PlainCoursingOrderIterator {
    stage: Stage,
    current_bell: usize,
    is_going_down: bool,
}

impl PlainCoursingOrderIterator {
    pub fn new(stage: Stage) -> PlainCoursingOrderIterator {
        PlainCoursingOrderIterator {
            stage,
            current_bell: ((stage.as_usize() + 1) & !1) - 2,
            is_going_down: true,
        }
    }
}

impl CoursingOrderIterator for PlainCoursingOrderIterator {
    fn next(&mut self) -> Bell {
        if self.stage == Stage::TWO {
            return Bell::from(1);
        }

        let current_bell = self.current_bell;

        if self.is_going_down {
            self.current_bell -= 2;

            if self.current_bell == 0 {
                self.is_going_down = false;
                self.current_bell = 1;
            }
        } else {
            self.current_bell += 2;

            if self.current_bell >= self.stage.as_usize() {
                self.is_going_down = true;
                self.current_bell = (self.stage.as_usize() + 1 & !1) - 2;
            }
        }

        Bell::from(current_bell)
    }

    fn length(&self) -> usize {
        self.stage.as_usize() - 1
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        first_plain_bob_lead_head, BasicCoursingOrderIterator, Bell, Change, CoursingOrder,
        CoursingOrderIterator, LeadheadCoursingOrderIterator, PlainCoursingOrderIterator, Stage,
        Transposition,
    };

    use crate::coursing_order::ZigZagIterator;

    #[test]
    fn zig_zag_iterator() {
        for s in -20..20 {
            // Test upwards
            let mut iter = ZigZagIterator::new(s, s + 1);

            assert_eq!(iter.next(), Some(s));

            for i in 1..100 {
                assert_eq!(iter.next(), Some(s + i));
                assert_eq!(iter.next(), Some(s - i));
            }

            // Test downwards
            let mut iter = ZigZagIterator::new(s, s - 1);

            assert_eq!(iter.next(), Some(s));

            for i in 1..100 {
                assert_eq!(iter.next(), Some(s - i));
                assert_eq!(iter.next(), Some(s + i));
            }
        }
    }

    #[test]
    #[should_panic]
    fn seek_safe_panic() {
        let co = &CoursingOrder::from_slice(Change::from("7654823").slice());

        let mut iter = BasicCoursingOrderIterator::new(&co);

        iter.seek_safe(Bell::from('0'));
    }

    #[test]
    fn seek_safe() {
        for string in &["2453687", "680972345", "5432876"] {
            let co = &CoursingOrder::from_slice(Change::from(*string).slice());

            let mut iter = BasicCoursingOrderIterator::new(&co);

            let mut chars = string.chars();

            chars.next();
            iter.next();

            chars.next();
            iter.next();

            iter.seek_safe(Bell::from(chars.next().unwrap()));

            assert_eq!(iter.next(), Bell::from(chars.next().unwrap()));
        }
    }

    #[test]
    fn plain_bob_lead_head() {
        for lh in &[
            "1342",
            "13524",
            "135264",
            "1352746",
            "13527486",
            "135274968",
            "1352749608",
            "13527496E80",
            "13527496E8T0",
        ] {
            let lh_change = Change::from(*lh);

            assert_eq!(first_plain_bob_lead_head(lh_change.stage()), lh_change);
        }
    }

    #[test]
    fn plain_iterator() {
        for order in &[
            "324",
            "5324",
            "53246",
            "753246",
            "7532468",
            "97532468",
            "975324680",
        ] {
            let stage = Stage::from(order.chars().count() + 1);

            let mut a = PlainCoursingOrderIterator::new(stage);
            let mut b = order.chars().cycle();

            for _ in 0..100 {
                let l = a.next().as_char();
                let r = b.next().unwrap();

                assert_eq!(l, r);
            }

            assert_eq!(a.length(), stage.as_usize() - 1);
        }
    }

    #[test]
    fn debug_print() {
        for order in &["8753462", "98762453", "5324", "65342", "8657234", "2"] {
            assert_eq!(
                format!("{:?}", CoursingOrder::from(*order)),
                format!("<{}>", order)
            );
        }
    }

    #[test]
    fn basic_iterator() {
        for order in &["8753462", "98762453", "5324", "65342", "8657234", "2"] {
            let co = CoursingOrder::from(*order);

            assert_eq!(BasicCoursingOrderIterator::new(&co).collect(), co);
        }
    }

    #[test]
    fn courseheads() {
        for s in 2..20 {
            let stage = Stage::from(s);

            assert_eq!(
                PlainCoursingOrderIterator::new(stage).to_coursehead(),
                Change::rounds(stage)
            );
        }

        for coursehead in &["12", "132456", "17364528", "17654328"] {
            let ch = Change::from(*coursehead);

            assert_eq!(CoursingOrder::from_leadhead(&ch).to_coursehead(), ch);
            assert_eq!(LeadheadCoursingOrderIterator::new(&ch).to_coursehead(), ch);
        }
    }

    #[test]
    fn leadhead_iterator() {
        for (lh, order) in &[
            ("12", "2"),
            ("12345", "5324"),
            ("15432", "5324"),
            ("4567123", "723165"),
            ("12348765", "8324756"),
            ("1209876543", "029753468"),
        ] {
            assert_eq!(
                LeadheadCoursingOrderIterator::new(&Change::from(*lh)).collect(),
                CoursingOrder::from(*order)
            );
        }
    }

    #[test]
    fn canonical_strings() {
        for (order, canon) in &[
            ("087953246", "CO: <87953246> runs: 65432s(H) 0987s(H)"),
            ("097246538", "CO: <97246538> runs: 23456s(B) 7890s(B)"),
            (
                "TE976824530",
                "CO: <E976824530> runs: 2345s(H) 9876s(H) 90ETs(B)",
            ),
            ("029753468", "CO: <29753468> runs: 09876543s(B)"),
            ("8753462", "CO: <753462> runs: 76543s(B)"),
            ("8645327", "CO: <645327> no runs. 87s"),
        ] {
            assert_eq!(CoursingOrder::from(*order).canonical_string(), *canon);
        }
    }
}
