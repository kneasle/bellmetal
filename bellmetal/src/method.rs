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
        Call::new (notation, PlaceNotation::from_multiple_string (string, stage))
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
        Change::from_iterator (self.lead_end_iterator ())
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

    pub fn is_lead_end_variant_of (&self, other : &Method) -> bool {
        if self.lead_length () != other.lead_length () {
            return false;
        }

        let mut own_iterator = self.place_notation.iter ().rev ().peekable ();
        let mut others_iterator = other.place_notation.iter ().rev ().peekable ();

        // Pop the lead end PNs
        own_iterator.next ();
        others_iterator.next ();

        while own_iterator.peek () != None {
            if own_iterator.next () != others_iterator.next () {
                return false;
            }
        }

        true
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
mod call_tests {
    use crate::{
        Call,
        Stage,
        PlaceNotation
    };

    #[test]
    #[should_panic]
    fn different_pn_stages () {
        Call::new (
            '-',
            vec![
                PlaceNotation::from_string ("14", Stage::MAJOR),
                PlaceNotation::from_string ("x", Stage::MINOR)
            ]
        );
    }

    #[test]
    #[should_panic]
    fn empty_pn () {
        Call::from_place_notation_string ('-', "", Stage::MAJOR);
    }
}

#[cfg(test)]
mod method_tests {
    use crate::{
        Method, Call, Stage, Change, PlaceNotation
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
    fn lead_end_variant () {
        for (a, b, stage_a, stage_b, exp) in &[
            ("1.5.1.3.2", "1.5.1.3.1", Stage::DOUBLES, Stage::DOUBLES, true),
            ("1.5.1.3.2", "1.5.1.3.1", Stage::DOUBLES, Stage::MINOR, false),
            ("x30x14x50x16x1270x38x14x50x16x90,12", "x30x14x50x16x1270x38x14x50x16x90,12", Stage::ROYAL, Stage::ROYAL, true),
            ("x1x1x1,2", "x1x1x1,1", Stage::MINOR, Stage::MINOR, true)
        ] {
            assert_eq! (
                Method::from_str ("A", a, *stage_a).is_lead_end_variant_of (
                    &Method::from_str ("B", b, *stage_b)
                ),
                *exp
            );
        }
    }

    #[test]
    fn lead_head_after_call () {
        assert_eq! (
            Method::from_str ("No Name", "7.1.7.1.7.1.7,127", Stage::TRIPLES).lead_head_after_call (
                &Call::from_place_notation_string ('-', "147", Stage::TRIPLES)
            ),
            Change::from ("1235746")
        )
    }

    #[test]
    fn partial () {
        assert_eq! (
            Method::partial (
                "Partial Method", 
                PlaceNotation::from_multiple_string ("x30", Stage::ROYAL),
                Change::from ("1352749608"),
                PlaceNotation::from_string ("12", Stage::ROYAL)
            ).plain_lead.to_string (),
            "1234567890\n2143658709\n1246385079\n1357294068\n3152749608\n1325476980"
        );
    }

    #[test]
    fn from_string () {
        for (pns, lh) in &[
            ("7.1.7.1.7.1.7,127", "1352746"), // Plain Bob Triples
            ("x3x4x25x36x47x58x69x70x8x9x0xE,2", "157392E4T608"), // Camb S Max
            ("3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", "4623751"), // Scientific Triples
            ("3,1.9.1.5.1", "126849375") // Little Grandsire Caters
        ] {
            assert_eq! (
                Method::from_str ("No Name", pns, Stage::from (lh.len ())),
                Method::from_string ("No Name".to_string (), pns, Stage::from (lh.len ()))
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
