use crate::{
    Stage, Bell, Place,
    Change, ChangeAccumulator,
    PlaceNotation,
    Touch, TouchIterator,
    Transposition
};




pub struct Call {
    pub place_notations : Vec<PlaceNotation>,
    pub transposition : Change,
    pub notation : char,
    pub stage : Stage
}

impl Call {
    pub fn from_place_notation_string (notation : char, string : &str, stage : Stage) -> Call {
        let place_notations = PlaceNotation::from_multiple_string (string, stage);

        if place_notations.len () == 0 {
            panic! ("Can't have a call with empty place notation array");
        }

        Call {
            transposition : PlaceNotation::overall_transposition (&place_notations),
            place_notations : place_notations,
            notation : notation,
            stage : stage
        }
    }

    pub fn new (notation : char, place_notations : Vec<PlaceNotation>) -> Call {
        if place_notations.len () == 0 {
            panic! ("Can't have a call with empty place notation array");
        }

        let stage = {
            let mut stage = None;

            for pn in &place_notations {
                match stage {
                    None => { stage = Some (pn.stage); }
                    Some (s) => { assert_eq! (pn.stage, s); }
                }
            }

            stage.unwrap ()
        };
        
        Call {
            transposition : PlaceNotation::overall_transposition (&place_notations),
            place_notations : place_notations,
            notation : notation,
            stage : stage
        }
    }
}




pub struct Method<'a> {
    pub name : &'a str,
    pub stage : Stage,
    
    pub place_notation : Vec<PlaceNotation>,
    pub plain_lead : Touch
}

impl<'a> Method<'a> {
    pub fn lead_length (&'a self) -> usize {
        self.plain_lead.length
    }

    pub fn lead_head (&'a self) -> &'a Change {
        &self.plain_lead.leftover_change
    }

    pub fn lead_end (&'a self) -> Change {
        let mut vec : Vec<Bell> = Vec::with_capacity (self.stage.as_usize ());
        
        for b in self.plain_lead.row_at (self.plain_lead.length - 1).slice () {
            vec.push (*b);
        }

        Change::new (vec)
    }

    pub fn lead_head_after_call (&'a self, call : &Call) -> Change {
        self.lead_end ().multiply (&call.transposition)
    }
}

impl Method<'_> {
    pub fn new<'a> (name : &'a str, place_notation : Vec<PlaceNotation>) -> Method {
        assert! (place_notation.len () > 0);

        Method {
            name : name,
            stage : place_notation [0].stage,
            plain_lead : Touch::from (&place_notation [..]),
            place_notation : place_notation
        }
    }

    pub fn from_string<'a> (name : &'a str, place_notation_str : &'a str, stage : Stage) -> Method<'a> {
        Method::new (name, PlaceNotation::from_multiple_string (place_notation_str, stage))
    }
}






pub struct SingleMethodTouchIterator<'a> {
    method : &'a Method<'a>,
    call_types : &'a [Call],
    call_list : &'a [usize],

    lead_head_accumulator : ChangeAccumulator,

    lead_index : usize,    // How many leads through the touch we are
    sub_lead_index : usize, // How many bells through the lead we are

    ruleoff_index : usize, // How many ruleoffs have been read

    bells_per_lead : usize // This takes 2 function calls to calculate so is worth caching
}

impl<'a> SingleMethodTouchIterator<'a> {
    fn is_finished (&'a self) -> bool {
        self.lead_index >= self.call_list.len ()
    }
}

impl SingleMethodTouchIterator<'_> {
    pub fn new<'a> (method : &'a Method, call_types : &'a [Call], call_list : &'a [usize]) -> SingleMethodTouchIterator<'a> {
        SingleMethodTouchIterator {
            method : method,
            call_types : call_types,
            call_list : call_list,

            lead_head_accumulator : ChangeAccumulator::new (method.stage),
            lead_index : 0,
            sub_lead_index : 0,
            ruleoff_index : 1,

            bells_per_lead : method.lead_length () * method.stage.as_usize ()
        }
    }
}

impl<'a> TouchIterator for SingleMethodTouchIterator<'a> {
    fn length (&self) -> usize {
        self.method.lead_length () * self.call_list.len ()
    }

    fn number_of_ruleoffs (&self) -> usize {
        self.call_list.len ()
    }

    fn stage (&self) -> Stage {
        self.method.stage
    }

    fn next_bell (&mut self) -> Option<Bell> {
        if self.is_finished () {
            return None;
        }

        let bell = self.lead_head_accumulator.total ().bell_at (
            Place::from (self.method.plain_lead.bell_at (self.sub_lead_index).as_usize ())
        );

        self.sub_lead_index += 1;

        if self.sub_lead_index == self.bells_per_lead {
            if self.call_list [self.lead_index] == 0 {
                self.lead_head_accumulator.accumulate (self.method.lead_head ());
            } else {
                self.lead_head_accumulator.accumulate (
                    &self.method.lead_head_after_call (
                        &self.call_types [self.call_list [self.lead_index] - 1]
                    )
                );
            }

            self.lead_index += 1;
            self.sub_lead_index = 0;
        }

        Some (bell)
    }

    fn next_ruleoff (&mut self) -> Option<usize> {
        if self.ruleoff_index >= self.call_list.len () {
            return None;
        }

        let v = self.method.lead_length () * self.ruleoff_index - 1;

        self.ruleoff_index += 1;

        Some (v)
    }

    fn reset (&mut self) {
        self.lead_head_accumulator.reset ();

        self.lead_index = 0;
        self.sub_lead_index = 0;
        self.ruleoff_index = 1;
    }

    fn leftover_change (&self) -> Change {
        if !self.is_finished () {
            panic! ("Can't generate leftover_change until the iterator is finished");
        }

        self.lead_head_accumulator.total ().clone ()
    }
}






#[cfg(test)]
mod tests {
    use crate::{
        Method,
        Stage, 
        Change
    };

    #[test]
    fn lead_lengths () {
        assert_eq! (
            Method::from_string ("Plain Bob Triples", "7.1.7.1.7.1.7,127", Stage::TRIPLES).lead_length (),
            14
        );

        assert_eq! (
            Method::from_string (
                "Cambridge Surprise Maximus",
                "x3Tx14x125Tx36x147Tx58x169Tx70x18x9Tx10xET,12",
                Stage::MAXIMUS
            ).lead_length (),
            48
        );
    }

    #[test]
    fn lead_ends () {
        assert_eq! (
            Method::from_string ("Plain Bob Triples", "7.1.7.1.7.1.7,127", Stage::TRIPLES).lead_end (),
            Change::from ("1325476")
        );

        assert_eq! (
            Method::from_string (
                "Cambridge Surprise Maximus",
                "x3Tx14x125Tx36x147Tx58x169Tx70x18x9Tx10xET,12",
                Stage::MAXIMUS
            ).lead_end (),
            Change::from ("1537294E6T80")
        );
    }

    #[test]
    fn lead_heads () {
        assert_eq! (
            *Method::from_string ("Plain Bob Triples", "7.1.7.1.7.1.7,127", Stage::TRIPLES).lead_head (),
            Change::from ("1352746")
        );

        assert_eq! (
            *Method::from_string (
                "Cambridge Surprise Maximus",
                "x3Tx14x125Tx36x147Tx58x169Tx70x18x9Tx10xET,12",
                Stage::MAXIMUS
            ).lead_head (),
            Change::from ("157392E4T608")
        );
    }
}
