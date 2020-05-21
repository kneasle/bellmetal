use crate::{
    Touch,
    Method,
    Call,
    ChangeAccumulator,
    TouchIterator
};

use std::collections::{ HashMap, HashSet };

pub fn one_part_spliced_touch (
    methods : &[(&str, &Method)], calls : &[(char, Call)],
    string : &str
) -> Touch {
    // Generate hashmaps and vectors from the arrays given
    let mut method_hashmap : HashMap<&str, &Method> = HashMap::with_capacity (methods.len ());
    let mut legit_method_starts : HashSet<char> = HashSet::with_capacity (methods.len ());
    let mut max_method_length = 0;

    for (notation, method) in methods.iter () {
        method_hashmap.insert (notation, method);

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
    let mut methods : Vec<(String, &Method)> = Vec::with_capacity (string.len ());
    let mut calls : Vec<Option<&Call>> = Vec::with_capacity (string.len ());

    {
        let mut partial_method_name = String::with_capacity (max_method_length);
        let mut has_consumed_call = true; // A hack to stop it adding an erraneous call at the start

        for c in string.chars () {
            if partial_method_name.len () == 0 { // We're between method names
                match call_hashmap.get (&c) {
                    Some (call) => {
                        if !has_consumed_call {
                            calls.push (Some (call));

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
                            calls.push (None);
                        }
                    }
                }
            }

            has_consumed_call = false;

            partial_method_name.push (c);

            match method_hashmap.get (&partial_method_name [..]) {
                Some (x) => {
                    methods.push ((partial_method_name.clone (), *x));

                    partial_method_name.clear ();
                }
                None => { }
            }
        }

        if !has_consumed_call {
            calls.push (None);
        }

        assert_eq! (partial_method_name.len (), 0);
        assert_eq! (methods.len (), calls.len ());
    }

    one_part_spliced_touch_from_indices (&methods [..], &calls [..])
}

fn one_part_spliced_touch_from_indices (
    methods : &[(String, &Method)], calls : &[Option<&Call>],
) -> Touch {
    // There should be at least one method otherwise the behaviour is undefined
    assert! (methods.len () > 0);
    assert! (methods.len () == methods.len ());

    // Find the stage of the touch
    let stage = methods [0].1.stage;

    for m in 1..methods.len () {
        assert_eq! (stage, methods [m].1.stage);
    }

    // Find the length of the touch
    let mut length = 0;

    for m in methods.iter () {
        length += m.1.lead_length ();
    }

    // Find the number of calls used
    let num_calls = calls.iter ().filter (|i| i.is_none ()).count ();
    let num_method_splices = methods.windows (2)
        .filter (|pair| pair [0] != pair [1])
        .count ();

    // Generate the touch
    let mut lead_head_accumulator = ChangeAccumulator::new (stage);
    let mut touch = Touch::with_capacity (stage, length, methods.len (), num_calls, num_method_splices);

    for i in 0..methods.len () {
        let (name, method) = &methods [i];

        if i == 0 || methods [i - 1] != methods [i] {
            touch.add_method_name (touch.length, name);
        }

        touch.append_iterator (&method.plain_lead.iter ().transfigure (lead_head_accumulator.total ()));

        if let Some (call) = calls [i] {
            touch.add_call (touch.length - 1, call.notation);

            lead_head_accumulator.accumulate_iterator (
                method.lead_head_after_call_iterator (
                    &call
                )
            );
        } else {
            lead_head_accumulator.accumulate (method.lead_head ());
        }
    }

    touch
}


#[cfg(test)]
mod tests {
    use crate::{ Method, Call, Stage, TouchIterator, one_part_spliced_touch, DefaultScoring };

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
}
