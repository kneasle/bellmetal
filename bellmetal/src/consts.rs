use crate::Number;

pub const MAX_STAGE : usize = 64;
pub static BELL_NAMES : &str = "1234567890ETABCDFGHJKLMNPRSUVWXYZ";

pub fn is_bell_name (c : char) -> bool {
    ((c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z')) && c != 'I' && c != 'O' && c != 'Q'
}

pub fn name_to_number (name : char) -> Number {
    let mut i = 0 as Number;

    for c in BELL_NAMES.chars () {
        if c == name {
            return i;
        }

        i += 1;
    }

    panic! ("Unknown bell name '{}'.", name);
}
