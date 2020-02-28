use crate::{
    Touch, TransfiguredTouchIterator,
    Method,
    Call,
    ChangeAccumulator
};

pub fn one_part_spliced_touch (
    methods : &[Method], calls : &[Call],
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
        let bristol = Method::from_string (
            "Bristol Surprise Major", "-58-14.58-58.36.14-14.58-14-18,18", Stage::MAJOR);
        let plain_bob = Method::from_string (
            "Plain Bob Major", "-18-18-18-18,12", Stage::MAJOR);
        let cornwall = Method::from_string (
            "Cornwall Surprise Major", "-56-14-56-38-14-58-14-58,18", Stage::MAJOR);
        let cambridge = Method::from_string (
            "Cambridge Surprise Major", "-38-14-1258-36-14-58-16-78,12", Stage::MAJOR);
        let lessness = Method::from_string (
            "Lessness Surprise Major", "-38-14-56-16-12-58-14-58,12", Stage::MAJOR);

        let bob = Call::from_place_notation_string ('-', "14", Stage::MAJOR);

        let touch = one_part_spliced_touch (
            &[bristol, plain_bob, cornwall, cambridge, lessness], &[bob],
            &[0, 1, 2, 3, 0, 4, 0], &[1, 1, 0, 1, 1, 1, 1]
        );
        
        // Assuming that it can't screw up and produce exactly the right number of 4-bell runs
        assert_eq! (touch.number_of_4_bell_runs (), (11, 12));
    }
}
