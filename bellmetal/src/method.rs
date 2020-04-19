use crate::{
    Stage, Bell,
    Change, ChangeAccumulator,
    PlaceNotation,
    Touch, Row,
    Transposition, MultiplicationIterator
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



#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Method {
    pub name : String,
    pub stage : Stage,

    pub place_notation : Vec<PlaceNotation>,
    pub plain_lead : Touch
}

impl Method {
    pub fn lead_length (&self) -> usize {
        self.plain_lead.length
    }

    pub fn lead_head (&self) -> &Change {
        &self.plain_lead.leftover_change
    }

    pub fn lead_end (&self) -> Change {
        let mut vec : Vec<Bell> = Vec::with_capacity (self.stage.as_usize ());

        for b in self.plain_lead.row_at (self.plain_lead.length - 1).slice () {
            vec.push (*b);
        }

        Change::new (vec)
    }

    pub fn lead_end_row<'a> (&'a self) -> Row<'a> {
        self.plain_lead.row_at (self.plain_lead.length - 1)
    }

    pub fn lead_end_slice<'a> (&'a self) -> &'a [Bell] {
        self.plain_lead.slice_at (self.plain_lead.length - 1)
    }

    pub fn lead_end_iterator<'a> (&'a self) -> std::iter::Cloned<std::slice::Iter<'a, Bell>> {
        self.lead_end_slice ().iter ().cloned ()
    }

    pub fn lead_head_after_call (&self, call : &Call) -> Change {
        self.lead_end ().multiply (&call.transposition)
    }

    pub fn lead_head_after_call_iterator<'a> (&'a self, call : &'a Call) -> impl Iterator<Item = Bell> + 'a {
        MultiplicationIterator::new (self.lead_end_slice (), call.transposition.iter ())
    }

    pub fn inverted (&self, new_name : &str) -> Method {
        Method {
            name : new_name.to_string (),
            stage : self.stage,
            plain_lead : self.plain_lead.inverted (),
            place_notation : self.place_notation.iter ().map (|x| x.reversed ()).collect ()
        }
    }
}

impl Method {
    pub fn new (name : String, place_notation : Vec<PlaceNotation>) -> Method {
        assert! (place_notation.len () > 0);

        Method {
            name : name,
            stage : place_notation [0].stage,
            plain_lead : Touch::from (&place_notation [..]),
            place_notation : place_notation
        }
    }

    pub fn partial (
        name : &str, place_notations : Vec<PlaceNotation>,
        lead_head : Change, lead_end_notation : PlaceNotation
    ) -> Method {
        let stage = lead_head.stage ();
        let lead_end = lead_head.multiply_iterator (lead_end_notation.iter ());

        let mut changes = Vec::with_capacity (place_notations.len () * 2);

        let mut acc = ChangeAccumulator::new (stage);

        for pn in place_notations {
            changes.push (acc.total ().clone ());
            acc.accumulate_iterator (pn.iter ());
        }

        changes.push (acc.total ().clone ());

        for i in (0..changes.len ()).rev () {
            changes.push (lead_end.multiply (&changes [i]));
        }

        Method {
            name : name.to_string (),
            stage : stage,
            plain_lead : Touch::from_changes (&changes [..], lead_head),
            place_notation : Vec::with_capacity (0)
        }
    }

    pub fn from_str (name : &str, place_notation_str : &str, stage : Stage) -> Method {
        Method::new (name.to_string (), PlaceNotation::from_multiple_string (place_notation_str, stage))
    }

    pub fn from_string (name : String, place_notation_str : &str, stage : Stage) -> Method {
        Method::new (name, PlaceNotation::from_multiple_string (place_notation_str, stage))
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
            Method::from_str ("Plain Bob Triples", "7.1.7.1.7.1.7,127", Stage::TRIPLES).lead_length (),
            14
        );

        assert_eq! (
            Method::from_str (
                "Cambridge Surprise Maximus",
                "x3Tx14x125Tx36x147Tx58x169Tx70x18x9Tx10xET,12",
                Stage::MAXIMUS
            ).lead_length (),
            48
        );
    }

    #[test]
    fn lead_ends () {
        for (pns, lh) in &[
            ("7.1.7.1.7.1.7,127", "1325476"), // Plain Bob Triples
            ("x3x4x25x36x47x58x69x70x8x9x0xE,2", "1537294E6T80"), // Camb S Max
            ("3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", "6432571"), // Scientific Triples
            ("3,1.9.1.5.1", "162483957") // Little Grandsire Caters
        ] {
            assert_eq! (
                Method::from_str ("No Name", pns, Stage::from (lh.len ())).lead_end (),
                Change::from (*lh)
            );
        }
    }

    #[test]
    fn lead_heads () {
        for (pns, lh) in &[
            ("7.1.7.1.7.1.7,127", "1352746"), // Plain Bob Triples
            ("x3x4x25x36x47x58x69x70x8x9x0xE,2", "157392E4T608"), // Camb S Max
            ("3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", "4623751"), // Scientific Triples
            ("3,1.9.1.5.1", "126849375") // Little Grandsire Caters
        ] {
            assert_eq! (
                *Method::from_str ("No Name", pns, Stage::from (lh.len ())).lead_head (),
                Change::from (*lh)
            );
        }
    }

    #[test]
    fn inversion () {
        for (pns, lh) in &[
            ("7.1.7.1.7.1.7,127", "2416357"), // Plain Bob Triples
            ("x3x4x25x36x47x58x69x70x8x9x0xE,2", "537192E4068T"), // Camb S Max
            ("3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", "7315624") // Scientific Triples
        ] {
            assert_eq! (
                *Method::from_str ("No Name", pns, Stage::from (lh.len ())).inverted ("Enam On").lead_head (),
                Change::from (*lh)
            );
        }
    }
}
