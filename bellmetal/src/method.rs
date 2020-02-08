use crate::types::{ Stage, Bell };
use crate::change::{ Change };
use crate::place_notation::PlaceNotation;
use crate::touch::Touch;
use crate::transposition::Transposition;




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
    plain_lead : Touch
}

impl<'a> Method<'a> {
    pub fn lead_head (&'a self) -> &'a Change {
        &self.plain_lead.leftover_change
    }

    pub fn lead_end (&'a self) -> Change {
        let mut vec : Vec<Bell> = Vec::with_capacity (self.stage.as_usize ());
        
        for b in self.plain_lead.row_at (self.plain_lead.length).slice () {
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






#[cfg(test)]
mod tests {
    use crate::method::Method;
    use crate::types::Stage;
    use crate::change::Change;

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
