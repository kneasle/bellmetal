use crate::{Bell, Mask, MaskMethods, Number, Touch, Transposition};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum WrapType {
    NoWrap,
    FullCyclic,
    FixedTrebleCyclic,
}

impl Default for WrapType {
    #[inline]
    fn default() -> Self {
        WrapType::NoWrap
    }
}

pub fn run_length_from_iter(i: impl Iterator<Item = Bell>) -> usize {
    let mut iter = i.peekable();

    if iter.peek() == None {
        return 0;
    }

    let mut last = iter.next().unwrap().as_i32();
    let mut i = 1;

    for b in iter {
        let diff = b.as_i32() - last;

        if diff != -1 && diff != 1 {
            break;
        }

        last = b.as_i32();

        i += 1;
    }

    i
}

fn run_length_of_slice_front(slice: &[Bell]) -> usize {
    run_length_from_iter(slice.iter().copied())
}

fn run_length_of_slice_back(slice: &[Bell]) -> usize {
    run_length_from_iter(slice.iter().copied().rev())
}

fn run_length_to_score(length: usize) -> usize {
    if length < 4 {
        return 0;
    }

    let x = length - 3;

    // Triangular numbers = n * (n + 1) / 2
    (x * (x + 1)) >> 1
}

pub trait MusicScoring {
    fn score_transposition(transposition: &impl Transposition) -> usize;
    fn highlight_transposition(transposition: &impl Transposition) -> Mask;

    fn score_touch(touch: &Touch) -> usize {
        touch
            .row_iterator()
            .map(|r| Self::score_transposition(&r))
            .sum()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct DefaultScoring();

impl MusicScoring for DefaultScoring {
    fn score_transposition(t: &impl Transposition) -> usize {
        let slice = t.slice();

        run_length_to_score(run_length_of_slice_front(slice))
            + run_length_to_score(run_length_of_slice_back(slice))
    }

    fn highlight_transposition(t: &impl Transposition) -> Mask {
        let slice = t.slice();
        let stage = slice.len();

        let run_length_front = run_length_of_slice_front(slice);
        let run_length_back = run_length_of_slice_back(slice);

        let mut mask = Mask::empty();

        if run_length_front >= 4 {
            for i in 0..run_length_front {
                mask.add(i as Number);
            }
        }

        if run_length_back >= 4 {
            for i in 0..run_length_back {
                mask.add((stage - 1 - i) as Number);
            }
        }

        mask
    }
}

pub fn cyclic_run_length_from_iter(i: impl Iterator<Item = Bell>, stage: i32) -> usize {
    // Get the first two items in the iterator, and return the right length if the iterator is
    // empty
    let mut iter = i.peekable();

    if iter.peek() == None {
        return 0;
    }

    let first = iter.next().unwrap().as_i32();

    if iter.peek() == None {
        return 1;
    }

    let second = iter.next().unwrap().as_i32();

    // Calculate the difference between the first two bells, and therefore what direction the run
    // is going in (ie is it descending or ascending?)
    let diff = if first == stage - 1 && second == 0 {
        1
    } else if first == 0 && second == stage - 1 {
        -1
    } else {
        second - first
    };

    // If this isn't 1, then there can't be a run
    if diff != -1 && diff != 1 {
        return 1;
    }

    // Continue reading values until a run stops and then return
    let mut length: usize = 2;

    for x in iter {
        if x.as_i32() != (((first + length as i32 * diff) % stage) + stage) % stage {
            break;
        }

        length += 1;
    }

    length
}

fn cyclic_run_length_of_slice_front(slice: &[Bell], stage: i32) -> usize {
    cyclic_run_length_from_iter(slice.iter().copied(), stage)
}

fn cyclic_run_length_of_slice_back(slice: &[Bell], stage: i32) -> usize {
    cyclic_run_length_from_iter(slice.iter().copied().rev(), stage)
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct FullCyclicScoring();

impl MusicScoring for FullCyclicScoring {
    fn score_transposition(t: &impl Transposition) -> usize {
        let slice = t.slice();
        let stage = slice.len() as i32;

        run_length_to_score(cyclic_run_length_of_slice_front(slice, stage))
            + run_length_to_score(cyclic_run_length_of_slice_back(slice, stage))
    }

    fn highlight_transposition(t: &impl Transposition) -> Mask {
        let slice = t.slice();
        let stage = slice.len();

        let run_length_front = cyclic_run_length_of_slice_front(slice, stage as i32);
        let run_length_back = cyclic_run_length_of_slice_back(slice, stage as i32);

        let mut mask = Mask::empty();

        if run_length_front >= 4 {
            for i in 0..run_length_front {
                mask.add(i as Number);
            }
        }

        if run_length_back >= 4 {
            for i in 0..run_length_back {
                mask.add((stage - 1 - i) as Number);
            }
        }

        mask
    }
}

#[cfg(test)]
mod tests {
    use crate::{Change, DefaultScoring, Transposition};

    #[test]
    fn music_scoring() {
        assert_eq!(Change::from("12347568").music_score::<DefaultScoring>(), 1);
        assert_eq!(Change::from("567894231").music_score::<DefaultScoring>(), 3);
        assert_eq!(
            Change::from("1234908765").music_score::<DefaultScoring>(),
            2
        );
        assert_eq!(
            Change::from("1234560978").music_score::<DefaultScoring>(),
            6
        );
        assert_eq!(
            Change::from("1234560987").music_score::<DefaultScoring>(),
            7
        );
        assert_eq!(Change::from("1234").music_score::<DefaultScoring>(), 2);
        assert_eq!(Change::from("15234").music_score::<DefaultScoring>(), 0);
        assert_eq!(
            Change::from("9876543210").music_score::<DefaultScoring>(),
            21
        );
        assert_eq!(
            Change::from("0987654321").music_score::<DefaultScoring>(),
            56
        );
    }
}
