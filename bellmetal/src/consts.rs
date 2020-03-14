use crate::Number;

pub const MAX_STAGE : usize = 64;
pub static BELL_NAMES : &str = "1234567890ETABCDFGHJKLMNPRSUVWXYZ";

static BELL_NAME_LOOKUP_TABLE : [i8; 91] = [
    -1, -1, -1, -1, -1, // 0..5
    -1, -1, -1, -1, -1, // 5..10
    -1, -1, -1, -1, -1, // 10..15
    -1, -1, -1, -1, -1, // 15..20
    -1, -1, -1, -1, -1, // 20..25
    -1, -1, -1, -1, -1, // 25..30
    -1, -1, -1, -1, -1, // 30..35
    -1, -1, -1, -1, -1, // 35..40
    -1, -1, -1, -1, -1, // 40..45
    -1, -1, -1,         // 45..48
    9,                 // 48 = '0'
    0, 1, 2, 3, 4, 5, 6, 7, 8, // 49..58 = '1'..'9'
    -1, -1,             // 58..60
    -1, -1, -1, -1, -1, // 60..65
    12, 13, 14, 15, 10, // 65..70 = 'A'-'D'
    16, 17, 18, -1, 19, // 70..75 = 'E'-'J'
    20, 21, 22, 23, -1, // 75..80 = 'K'-'O'
    24, -1, 25, 26, 11, // 80..85 = 'P'-'T'
    27, 28, 29, 30, 31, 32 // 85..91 = 'U'-'Z'
];


pub fn is_bell_name (c : char) -> bool {
    ((c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z')) && c != 'I' && c != 'O' && c != 'Q'
}

fn get_number (name : char) -> i8 {
    if name as usize >= BELL_NAME_LOOKUP_TABLE.len () {
        return -1;
    }

    BELL_NAME_LOOKUP_TABLE [name as usize]
}

pub fn name_to_number (name : char) -> Number {
    let n = get_number (name);
    
    if n == -1 {
        panic! ("Unknown bell name '{}'.", name);
    }

    n as Number
}

#[cfg(test)]
mod const_tests {
    use crate::{ BELL_NAMES, Bell, name_to_number };
    use crate::consts::get_number;

    #[test]
    fn lookup_table () {
        fn get_from_names (name : char) -> i8 {
            for (i, c) in BELL_NAMES.chars ().enumerate () {
                if c == name {
                    return i as i8;
                }
            }

            -1
        }

        for i in 0..127u8 {
            let c = i as char;

            print! ("{}", c);

            assert_eq! (get_from_names (c), get_number (c));
        }
    }

    #[test]
    fn char_conversion () {
        for c in BELL_NAMES.chars () {
            assert_eq! (Bell::from (name_to_number (c)).as_char (), c);
        }
    }
}
