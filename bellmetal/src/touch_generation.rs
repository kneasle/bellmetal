use crate::{
    Touch,
    Method,
    Call,
    Change,
    TouchIterator,
    Transposition
};

use std::collections::{ HashMap, HashSet };
use itertools::Itertools;
use std::iter::repeat;

pub fn single_method_touch (
    method : &Method,
    mnemonic : &str,
    calls : &[Vec<&Call>]
) -> Touch {
    one_part_spliced_touch_from_indices (repeat ((mnemonic, method)).take (calls.len ()), calls)
}

pub fn one_part_spliced_touch (
    methods : &[(&str, &Method)], calls : &[(char, Call)],
    string : &str
) -> Touch {
    // Generate hashmaps and vectors from the arrays given
    let mut method_hashmap : HashMap<&str, (&str, &Method)> = HashMap::with_capacity (methods.len ());
    let mut legit_method_starts : HashSet<char> = HashSet::with_capacity (methods.len ());
    let mut max_method_length = 0;

    for (notation, method) in methods.iter () {
        method_hashmap.insert (notation, (notation, method));

        legit_method_starts.insert (notation.chars ().next ().unwrap ());

        if notation.len () > max_method_length {
            max_method_length = notation.len ();
        }
    }

    let mut call_hashmap : HashMap<char, &Call> = HashMap::with_capacity (calls.len ());

    for (notation, call) in calls.iter () {
        call_hashmap.insert (*notation, &call);
    }

    // Parse the string
    let mut methods : Vec<(&str, &Method)> = Vec::with_capacity (string.len ());
    let mut calls : Vec<Vec<&Call>> = Vec::with_capacity (string.len ());

    {
        let mut partial_method_name = String::with_capacity (max_method_length);
        let mut has_consumed_call = true; // A hack to stop it adding an erraneous call at the start

        for c in string.chars () {
            if partial_method_name.len () == 0 { // We're between method names
                match call_hashmap.get (&c) {
                    Some (call) => {
                        if !has_consumed_call {
                            calls.push (vec! [call]);

                            has_consumed_call = true;

                            continue;
                        }
                    }
                    None => {
                        if !legit_method_starts.contains (&c) {
                            // Ignore padding characters between method names
                            continue;
                        }

                        if !has_consumed_call {
                            calls.push (vec! []);
                        }
                    }
                }
            }

            has_consumed_call = false;

            partial_method_name.push (c);

            match method_hashmap.get (&partial_method_name [..]) {
                Some (x) => {
                    methods.push (*x);

                    partial_method_name.clear ();
                }
                None => { }
            }
        }

        if !has_consumed_call {
            calls.push (vec! []);
        }

        assert_eq! (partial_method_name.len (), 0);
        assert_eq! (methods.len (), calls.len ());
    }

    one_part_spliced_touch_from_indices (methods.iter ().cloned (), &calls [..])
}

pub fn one_part_spliced_touch_from_indices<'a> (
    methods : impl Iterator<Item = (&'a str, &'a Method)> + Clone, calls : &[Vec<&Call>],
) -> Touch {
    // Find the stage and length of the touch (and make sure that methods is non-empty)
    let mut method_iter = methods.clone ();

    let first_method = method_iter.next ().expect ("Can't have a touch with no methods.").1;
    let stage = first_method.stage;
    let mut length = first_method.lead_length ();

    let mut num_methods = 1;
    for m in method_iter {
        assert_eq! (m.1.stage, stage);
        num_methods += 1;
        length += m.1.lead_length ();
    }

    // Find the number of calls used
    let num_calls = calls.iter ().filter (|i| i.len () == 0).count ();
    let num_method_splices = methods.clone ()
        .tuple_windows ()
        .filter (|(x, y)| x.1 != y.1)
        .count ();

    // Generate the touch
    let mut current_lead_head = Change::rounds (stage);
    let mut touch = Touch::with_capacity (stage, length, num_methods, num_calls, num_method_splices);

    macro_rules! process_lead {
        ($method : expr, $calls : expr) => {
            touch.append_iterator (
                &$method.get_lead_fragment_with_calls (
                    0, $method.lead_length (),
                    $calls.iter ().map (|x| *x)
                ).iter ().transfigure (&current_lead_head)
            );

            touch.leftover_change.copy_into (&mut current_lead_head);
        }
    };

    // Handle first lead as special case
    let mut iter = methods.zip (calls.iter ());

    let ((name, method), calls) = iter.next ().unwrap ();
        
    process_lead! (method, calls);
            
    touch.add_method_name (0, name);

    let mut last_method = method;

    // Now handle all remaining leads in a loop
    for ((name, method), calls) in iter {
        if last_method != method {
            touch.add_method_name (touch.length, name);
        }

        process_lead! (method, calls);

        last_method = method;
    }

    touch
}


#[cfg(test)]
mod tests {
    use crate::{ Method, Call, Change, Stage, TouchIterator, one_part_spliced_touch, DefaultScoring };

    #[test]
    fn one_part_spliced () {
        let bristol = Method::from_str (
            "Bristol Surprise Major", "-58-14.58-58.36.14-14.58-14-18,18", Stage::MAJOR);
        let plain_bob = Method::from_str (
            "Plain Bob Major", "-18-18-18-18,12", Stage::MAJOR);
        let cornwall = Method::from_str (
            "Cornwall Surprise Major", "-56-14-56-38-14-58-14-58,18", Stage::MAJOR);
        let cambridge = Method::from_str (
            "Cambridge Surprise Major", "-38-14-1258-36-14-58-16-78,12", Stage::MAJOR);
        let lessness = Method::from_str (
            "Lessness Surprise Major", "-38-14-56-16-12-58-14-58,12", Stage::MAJOR);

        let bob = Call::lead_end_call_from_place_notation_string ('-', "14", Stage::MAJOR);

        let methods = [
            ("B", &bristol),
            ("P", &plain_bob),
            ("Co", &cornwall),
            ("Ca", &cambridge),
            ("E", &lessness)
        ];

        let calls = [
            ('-', bob)
        ];

        for (input_string, summary) in &[
            ("CoCa", "64 changes, true.  Score: 40. 7 4-bell runs (2f, 5b)"),
            ("   CoXXLDKJFLCa    ", "64 changes, true.  Score: 40. 7 4-bell runs (2f, 5b)"),
            ("B-P - \0Co Ca\t\n-B** X-E-B-", "208 changes, true.  Score: 71. 23 4-bell runs (11f, 12b)"),
            ("BBCa", "96 changes, false.  Score: 58. 11 4-bell runs (4f, 7b)")
        ] {
            let touch = one_part_spliced_touch (&methods, &calls, input_string);

            assert_eq! (touch.iter ().collect (), touch);

            // Assuming that it can't screw up and produce exactly the right summary string
            assert_eq! (touch.summary_string::<DefaultScoring> (), *summary);
        }
    }

    #[test]
    fn leftover_change_at_call () {
        let bristol = Method::from_str (
            "Bristol Surprise Major", "-58-14.58-58.36.14-14.58-14-18,18", Stage::MAJOR);

        let bob = Call::lead_end_call_from_place_notation_string ('-', "14", Stage::MAJOR);

        let methods = [("B", &bristol)];

        let calls = [('-', bob)];

        let touch = one_part_spliced_touch (&methods, &calls, "B-B-B-");

        assert_eq! (touch.leftover_change, Change::rounds (Stage::MAJOR));
    }
}
