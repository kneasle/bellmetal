use crate::{ Bell, Number, Touch, Transposition, Mask, MaskMethods };

pub fn run_length_from_iter (i : impl Iterator<Item = Bell>) -> usize {
    let mut iter = i.peekable ();

    if iter.peek () == None {
        return 0;
    }

    let mut last = iter.next ().unwrap ().as_i32 ();
    let mut i = 1;

    loop {
        if let Some (b) = iter.next () {
            let diff = b.as_i32 () - last;

            if diff != -1 && diff != 1 {
                break;
            }

            last = b.as_i32 ()
        } else {
            break;
        }
        
        i += 1;
    }

    i
}

fn run_length_of_slice_front (slice : &[Bell]) -> usize {
    run_length_from_iter (slice.iter ().copied ())
}

fn run_length_of_slice_back (slice : &[Bell]) -> usize {
    run_length_from_iter (slice.iter ().copied ().rev ())
}






pub trait MusicScoring {
    fn score_transposition (transposition : &impl Transposition) -> usize;
    fn highlight_transposition (transposition : &impl Transposition) -> Mask;

    fn score_touch (touch : &Touch) -> usize {
        touch.row_iterator ().map (|r| Self::score_transposition (&r)).sum ()
    }
}







pub struct DefaultScoring { }

fn run_length_to_score (length : usize) -> usize {
    if length < 4 {
        return 0;
    }

    let x = length - 3;

    // Triangular numbers = n * (n + 1) / 2
    (x * (x + 1)) >> 1
}

impl MusicScoring for DefaultScoring {
    fn score_transposition (t: &impl Transposition) -> usize {
        let slice = t.slice ();

        run_length_to_score (run_length_of_slice_front (slice))
            + run_length_to_score (run_length_of_slice_back (slice))
    }

    fn highlight_transposition (t : &impl Transposition) -> Mask {
        let slice = t.slice ();
        let stage = slice.len ();

        let run_length_front = run_length_of_slice_front (slice);
        let run_length_back = run_length_of_slice_back (slice);

        let mut mask = Mask::empty ();

        if run_length_front >= 4 {
            for i in 0..run_length_front {
                mask.add (i as Number);
            }
        }

        if run_length_back >= 4 {
            for i in 0..run_length_back {
                mask.add ((stage - 1 - i) as Number);
            }
        }

        mask
    }
}







#[cfg(test)]
mod tests {
    use crate::{ Change, Transposition, DefaultScoring };

    #[test]
    fn music_scoring () {
        assert_eq! (Change::from ("12347568").music_score::<DefaultScoring> (), 1);
        assert_eq! (Change::from ("567894231").music_score::<DefaultScoring> (), 3);
        assert_eq! (Change::from ("1234908765").music_score::<DefaultScoring> (), 2);
        assert_eq! (Change::from ("1234560978").music_score::<DefaultScoring> (), 6);
        assert_eq! (Change::from ("1234560987").music_score::<DefaultScoring> (), 7);
        assert_eq! (Change::from ("1234").music_score::<DefaultScoring> (), 2);
        assert_eq! (Change::from ("15234").music_score::<DefaultScoring> (), 0);
        assert_eq! (Change::from ("9876543210").music_score::<DefaultScoring> (), 21);
        assert_eq! (Change::from ("0987654321").music_score::<DefaultScoring> (), 56);
    }
}
