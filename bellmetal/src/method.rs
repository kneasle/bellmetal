use crate::types::Stage;
use crate::change::Change;
use crate::place_notation::PlaceNotation;
use crate::touch::Touch;

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
