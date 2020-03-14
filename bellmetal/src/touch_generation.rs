use crate::{
    Touch, TransfiguredTouchIterator,
    Method,
    Call,
    ChangeAccumulator
};

use std::collections::{ HashMap, HashSet };

pub fn one_part_spliced_touch (
    methods : &[(&str, &Method)], calls : &[(char, Call)],
    string : &str
) -> Touch {
    // Generate hashmaps and vectors from the arrays given
    let mut method_hashmap : HashMap<&str, usize> = HashMap::with_capacity (methods.len ());
    let mut method_list : Vec<&Method> = Vec::with_capacity (methods.len ());
    let mut legit_method_starts : HashSet<char> = HashSet::with_capacity (methods.len ());
    let mut max_method_length = 0;

    for (i, (notation, method)) in methods.iter ().enumerate () {
        method_list.push (method);
        method_hashmap.insert (notation, i);

        legit_method_starts.insert (notation.chars ().next ().unwrap ());

        if notation.len () > max_method_length {
            max_method_length = notation.len ();
        }
    }

    let mut call_hashmap : HashMap<char, usize> = HashMap::with_capacity (calls.len ());
    let mut call_list : Vec<&Call> = Vec::with_capacity (calls.len ());

    for (i, (notation, call)) in calls.iter ().enumerate () {
        call_list.push (call);
        call_hashmap.insert (*notation, i);
    }

    // Parse the string
    let mut method_indices : Vec<usize> = Vec::with_capacity (string.len ());
    let mut call_indices : Vec<usize> = Vec::with_capacity (string.len ());

    {
        let mut partial_method_name = String::with_capacity (max_method_length);
        let mut has_consumed_call = true; // A hack to stop it adding an erraneous call at the start

        for c in string.chars () {
            if partial_method_name.len () == 0 { // We're between method names
                match call_hashmap.get (&c) {
                    Some (x) => {
                        if !has_consumed_call {
                            call_indices.push (*x + 1);

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
                            call_indices.push (0);
                        }
                    }
                }
            }

            has_consumed_call = false;

            partial_method_name.push (c);

            match method_hashmap.get (&partial_method_name [..]) {
                Some (x) => {
                    method_indices.push (*x);

                    partial_method_name.clear ();
                }
                None => { }
            }
        }

        if !has_consumed_call {
            call_indices.push (0);
        }

        assert_eq! (partial_method_name.len (), 0);
        assert_eq! (method_indices.len (), call_indices.len ());
    }

    one_part_spliced_touch_from_indices (
        &method_list [..], &call_list [..],
        &method_indices [..], &call_indices [..]
    )
}

fn one_part_spliced_touch_from_indices (
    methods : &[&Method], calls : &[&Call],
    method_indices : &[usize], call_indices : &[usize]
) -> Touch {
    // There should be at least one method otherwise the behaviour is undefined
    assert! (methods.len () > 0);
    assert! (method_indices.len () == call_indices.len ());

    // Find the stage of the touch
    let stage = methods [0].stage;
    
    for m in 1..methods.len () {
        assert_eq! (stage, methods [m].stage);
    }

    // Find the length of the touch
    let mut length = 0;

    for i in method_indices.iter () {
        length += methods [*i].lead_length ();
    }

    // Generate the touch
    let mut lead_head_accumulator = ChangeAccumulator::new (stage);
    let mut touch = Touch::with_capacity (stage, length, method_indices.len ());

    for i in 0..method_indices.len () {
        let method = &methods [method_indices [i]];

        touch.append_iterator (
            &mut TransfiguredTouchIterator::new (
                lead_head_accumulator.total (),
                &method.plain_lead
            )
        );

        if call_indices [i] == 0 {
            lead_head_accumulator.accumulate (method.lead_head ());
        } else {
            lead_head_accumulator.accumulate_iterator (
                method.lead_head_after_call_iterator (
                    &calls [call_indices [i] - 1]
                )
            );
        }
    }

    touch
}


#[cfg(test)]
mod gen_tests {
    use crate::{ Method, Call, Stage, one_part_spliced_touch };

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

        let bob = Call::from_place_notation_string ('-', "14", Stage::MAJOR);

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

        // Assuming that it can't screw up and produce exactly the right number of 4-bell runs
        assert_eq! (
            one_part_spliced_touch (
                &methods, &calls [..], "CoCa"
            ).number_of_4_bell_runs (),
            (2, 5)
        );

        assert_eq! (
            one_part_spliced_touch (
                &methods, &calls [..], "   CoXXLDKJFLCa    "
            ).number_of_4_bell_runs (),
            (2, 5)
        );

        assert_eq! (
            one_part_spliced_touch (
                &methods, &calls [..], "B-P - \0Co Ca\t\n-B** X-E-B-"
            ).number_of_4_bell_runs (), 
            (11, 12)
        );
    }
}
