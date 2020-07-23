use crate::music_scoring::run_length_from_iter;
use crate::types::*;
use crate::{Change, MusicScoring};

pub trait Transposition {
    fn slice(&self) -> &[Bell];

    fn iter<'a>(&'a self) -> std::iter::Cloned<std::slice::Iter<'a, Bell>> {
        self.slice().iter().cloned()
    }

    fn naive_hash(&self) -> usize {
        let mut val = 0;
        let stage = self.slice().len();

        for b in self.slice() {
            val *= stage;
            val += b.as_usize();
        }

        val
    }

    fn stage(&self) -> Stage {
        Stage::from(self.slice().len())
    }

    fn bell_at(&self, place: Place) -> Bell {
        self.slice()[place.as_usize()]
    }

    fn place_of(&self, bell: Bell) -> Place {
        for (i, b) in self.slice().iter().enumerate() {
            if *b == bell {
                return Place::from(i);
            }
        }

        panic!(
            "Bell '{}' not found in <{}>",
            bell.as_char(),
            self.slice().iter().map(|x| x.as_char()).collect::<String>()
        );
    }

    fn inverse(&self) -> Change {
        let mut new_seq: Vec<Bell> = vec![Bell::from(0 as Number); self.stage().as_usize()];

        let slice = self.slice();

        for i in 0..slice.len() {
            new_seq[slice[i as usize].as_usize()] = Bell::from(i);
        }

        Change::new(new_seq)
    }

    fn parity(&self) -> Parity {
        let bells = self.slice();
        let stage = bells.len();

        let mut mask = Mask::empty();
        let mut bells_fixed = 0;

        let mut total_cycle_length = 0;

        while bells_fixed < stage {
            let mut bell = 0;

            while mask.get(bell) {
                bell += 1;
            }

            total_cycle_length += 1; // Make sure that the parity is correct

            while !mask.get(bell) {
                mask.add(bell);

                bell = bells[bell as usize].as_number();

                total_cycle_length += 1;
                bells_fixed += 1;
            }
        }

        match total_cycle_length & 1 {
            0 => Parity::Even,
            1 => Parity::Odd,
            _ => panic!("Unknown parity"),
        }
    }

    fn is_continuous_with<T: Transposition>(&self, other: T) -> bool {
        let a = self.slice();
        let b = other.slice();

        if a.len() != b.len() {
            return false;
        }

        let mut i = 0;

        while i < a.len() {
            if a[i] == b[i] {
                i += 1;
            } else if a[i] == b[i + 1] && b[i] == a[i + 1] {
                i += 2;
            } else {
                return false;
            }
        }

        true
    }

    // Music scoring (follows roughly what CompLib does, but IMO it makes long runs overpowered)
    fn music_score<T: MusicScoring>(&self) -> usize
    where
        Self: Sized,
    {
        T::score_transposition(self)
    }

    fn run_length_off_front(&self) -> usize {
        run_length_from_iter(self.slice().iter().copied())
    }

    fn run_length_off_back(&self) -> usize {
        run_length_from_iter(self.slice().iter().copied().rev())
    }

    // Useful change tests
    fn is_full_cyclic(&self) -> bool {
        let bells = self.slice();

        let stage = bells.len();

        if stage == 0 {
            return false;
        }

        let start = bells[0].as_usize();

        for (i, bell) in bells.iter().enumerate() {
            if bell.as_usize() != (start + i) % stage {
                return false;
            }
        }

        true
    }

    fn is_reverse_full_cyclic(&self) -> bool {
        let bells = self.slice();

        let stage = bells.len();

        if stage == 0 {
            return false;
        }

        let start = bells[0].as_usize() + stage;

        for (i, bell) in bells.iter().enumerate() {
            if bell.as_usize() != (start - i) % stage {
                return false;
            }
        }

        true
    }

    fn is_fixed_treble_cyclic(&self) -> bool {
        let bells = self.slice();

        let stage = bells.len();

        if stage <= 2 || bells[0].as_usize() != 0 {
            return false;
        }

        let start = bells[1].as_usize();

        for i in 0..stage - 1 {
            let expected_bell = if start + i >= stage {
                start + i - stage + 1
            } else {
                start + i
            };

            if bells[i + 1].as_usize() != expected_bell {
                return false;
            }
        }

        true
    }

    fn is_reverse_fixed_treble_cyclic(&self) -> bool {
        // This works the same way is 'is_fixed_treble_cyclic', but it iterates backwards
        // starting with the bell at the back

        let bells = self.slice();

        let stage = bells.len();

        if stage <= 2 || bells[0].as_usize() != 0 {
            return false;
        }

        let start = bells[stage - 1].as_usize();

        for i in 0..stage - 1 {
            let expected_bell = if start + i >= stage {
                start + i - stage + 1
            } else {
                start + i
            };

            if bells[stage - 1 - i].as_usize() != expected_bell {
                return false;
            }
        }

        true
    }

    fn is_backrounds(&self) -> bool {
        let bells = self.slice();
        let stage = bells.len();

        for (i, bell) in bells.iter().enumerate() {
            if bell.as_usize() != stage - 1 - i {
                return false;
            }
        }

        true
    }

    fn is_rounds(&self) -> bool {
        for (i, bell) in self.slice().iter().enumerate() {
            if bell.as_usize() != i {
                return false;
            }
        }

        true
    }

    fn inverted(&self) -> Change
    where
        Self: Sized,
    {
        let stage_minus_1 = self.slice().len() - 1;

        Change::new(
            self.slice()
                .iter()
                .rev()
                .map(|b| Bell::from(stage_minus_1 - b.as_usize()))
                .collect(),
        )
    }

    fn copy_into(&self, other: &mut Change)
    where
        Self: Sized,
    {
        other.overwrite_from(self);
    }

    // To string
    fn to_string(&self) -> String {
        let mut string = String::with_capacity(self.slice().len());

        for i in self.slice() {
            string.push(i.as_char());
        }

        string
    }

    // Pretty printing
    fn pretty_string<T: MusicScoring>(&self) -> String
    where
        Self: Sized,
    {
        let mut string = String::with_capacity(self.slice().len() * 3); // Seems a good length

        self.write_pretty_string::<T>(&mut string);

        string
    }

    fn write_pretty_string<T: MusicScoring>(&self, string: &mut String)
    where
        Self: Sized,
    {
        self.write_pretty_string_with_stroke::<T>(string, Stroke::Hand);
    }

    fn pretty_string_with_stroke<T: MusicScoring>(&self, stroke: Stroke) -> String
    where
        Self: Sized,
    {
        let mut string = String::with_capacity(self.slice().len() * 3); // Seems a good length

        self.write_pretty_string_with_stroke::<T>(&mut string, stroke);

        string
    }

    fn write_pretty_string_with_stroke<T: MusicScoring>(&self, string: &mut String, stroke: Stroke)
    where
        Self: Sized,
    {
        #[derive(PartialEq, Eq)]
        enum CharState {
            Normal,
            Musical,
            Undesirable,
        }

        let bells = self.slice();

        let stage = bells.len();

        let music_mask = T::highlight_transposition(self);

        let is_87_at_back = stage % 2 == 0
            && stroke == Stroke::Back
            && bells[stage - 2] == Bell::from(stage - 1)
            && bells[stage - 1] == Bell::from(stage - 2);

        let mut last_char_state = CharState::Normal;
        let mut last_char_colour = 0;

        let colours = ["97", "91", "96"];

        for (i, bell) in bells.iter().enumerate() {
            // Useful vars
            let char_colour = if bell.as_usize() == 0 {
                1
            } else if bell.as_usize() == stage - 1 {
                2
            } else {
                0
            };

            let char_state = if music_mask.get(i as u32) {
                CharState::Musical
            } else if is_87_at_back && i >= stage - 2 {
                CharState::Undesirable
            } else {
                CharState::Normal
            };

            // Push the escape codes
            if last_char_colour != char_colour || last_char_state != char_state {
                string.push_str("\x1b[");
                string.push_str(colours[char_colour]);
                string.push(';');
                string.push_str(match char_state {
                    CharState::Musical => "42",
                    CharState::Undesirable => "41",
                    CharState::Normal => "49",
                });
                string.push('m');
            }

            string.push(bell.as_char());

            last_char_state = char_state;
            last_char_colour = char_colour;
        }

        string.push_str("\x1b[0m"); // Always reset the formatting
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MultiplicationIterator<'a, T: Iterator<Item = Bell>> {
    lhs: &'a [Bell],
    rhs: T,
}

impl<T> MultiplicationIterator<'_, T>
where
    T: Iterator<Item = Bell>,
{
    pub fn new(lhs: &[Bell], rhs: T) -> MultiplicationIterator<T> {
        MultiplicationIterator { lhs, rhs }
    }
}

impl<'a, T> Iterator for MultiplicationIterator<'a, T>
where
    T: Iterator<Item = Bell>,
{
    type Item = Bell;

    fn next(&mut self) -> Option<Bell> {
        match self.rhs.next() {
            Some(b) => Some(self.lhs[b.as_usize()]),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rhs.size_hint()
    }
}
