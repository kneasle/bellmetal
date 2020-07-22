use crate::{
    Bell, Change, ChangeAccumulator, MultiplicationIterator, PlaceNotation, Row, Stage, Touch,
    Transposition,
};

use common_macros::hash_map;
use std::collections::HashMap;

pub const LEAD_END_LOCATION: &str = "LE";
pub const HALF_LEAD_LOCATION: &str = "HL";

#[derive(Hash, Debug)]
pub struct Call {
    pub place_notations: Vec<PlaceNotation>,
    pub transposition: Change,
    pub notation: char,
    pub location: String,
    pub stage: Stage,
}

impl Call {
    pub fn lead_end_call_from_place_notation_string(
        notation: char,
        string: &str,
        stage: Stage,
    ) -> Call {
        Call::new(
            notation,
            PlaceNotation::from_multiple_string(string, stage),
            LEAD_END_LOCATION,
        )
    }

    pub fn from_place_notation_string(
        notation: char,
        string: &str,
        stage: Stage,
        location: &str,
    ) -> Call {
        Call::new(
            notation,
            PlaceNotation::from_multiple_string(string, stage),
            location,
        )
    }

    pub fn new(notation: char, place_notations: Vec<PlaceNotation>, location: &str) -> Call {
        if place_notations.is_empty() {
            panic!("Can't have a call with empty place notation array");
        }

        let stage = {
            let mut stage = None;

            for pn in &place_notations {
                match stage {
                    None => {
                        stage = Some(pn.stage);
                    }
                    Some(s) => {
                        assert_eq!(pn.stage, s);
                    }
                }
            }

            stage.unwrap()
        };

        Call {
            transposition: PlaceNotation::overall_transposition(&place_notations),
            place_notations,
            notation,
            location: location.to_string(),
            stage,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Method {
    pub name: String,
    pub stage: Stage,

    pub place_notation: Vec<PlaceNotation>,
    pub plain_lead: Touch,

    location_map: HashMap<String, usize>,
}

impl Method {
    pub fn lead_length(&self) -> usize {
        self.plain_lead.length
    }

    pub fn add_location(&mut self, name: &str, index: usize) {
        self.location_map.insert(name.to_string(), index);
    }

    pub fn lead_head(&self) -> &Change {
        &self.plain_lead.leftover_change
    }

    pub fn lead_end(&self) -> Change {
        Change::from_iterator(self.lead_end_iterator())
    }

    pub fn lead_end_row(&self) -> Row {
        self.plain_lead.row_at(self.plain_lead.length - 1)
    }

    pub fn lead_end_slice(&self) -> &[Bell] {
        self.plain_lead.slice_at(self.plain_lead.length - 1)
    }

    pub fn lead_end_iterator<'a>(&'a self) -> std::iter::Cloned<std::slice::Iter<'a, Bell>> {
        self.lead_end_slice().iter().cloned()
    }

    pub fn lead_head_after_call(&self, call: &Call) -> Change {
        self.lead_end().multiply(&call.transposition)
    }

    pub fn lead_head_after_call_iterator<'a>(
        &'a self,
        call: &'a Call,
    ) -> impl Iterator<Item = Bell> + 'a {
        MultiplicationIterator::new(self.lead_end_slice(), call.transposition.iter())
    }

    pub fn get_lead_fragment_with_calls<'a>(
        &self,
        start_index: usize,
        end_index: usize,
        calls: impl IntoIterator<Item = &'a Call>,
    ) -> Touch {
        // Generate a map of which changes calls have been put on
        let iter = calls.into_iter();

        let call_map_capacity = iter.size_hint().1.unwrap_or(10);

        let mut call_map: Vec<(usize, &Call)> = Vec::with_capacity(call_map_capacity);

        for c in iter {
            if let Some(&i) = self.location_map.get(&c.location) {
                if i > start_index && i <= end_index {
                    call_map.push((i, c));
                }
            }
        }

        call_map.sort_by_key(|x| x.0);

        // Generate fragments
        #[derive(Debug, Hash)]
        enum Fragment<'b> {
            Plain(usize, usize),
            Call(&'b Call),
        }

        let mut fragments: Vec<Fragment<'a>> = Vec::with_capacity(call_map.len() * 2 + 1);

        let mut last_index = start_index;

        for (index, call) in call_map {
            fragments.push(Fragment::Plain(
                last_index,
                index - call.place_notations.len(),
            ));
            fragments.push(Fragment::Call(call));

            last_index = index;
        }

        if last_index != self.lead_length() {
            fragments.push(Fragment::Plain(last_index, end_index));
        }

        // Generate the touch
        let mut current_change = self.plain_lead.row_at(start_index).inverse();

        let mut touch =
            Touch::with_capacity(self.stage, self.lead_length(), 1, call_map_capacity, 1);

        for f in fragments {
            match f {
                Fragment::Plain(start, end) => {
                    touch.append_bell_iterator(
                        current_change.transfigure_iterator(
                            self.plain_lead
                                .fragment_bell_iterator(start, end + 1)
                                .cloned(),
                        ),
                    );
                }
                Fragment::Call(call) => {
                    touch.add_call(touch.length, call.notation);
                    touch.extend_with_place_notation(&call.place_notations);
                }
            }

            if touch.length < self.plain_lead.length {
                touch.leftover_change.multiply_inverse_into(
                    &self.plain_lead.row_at(touch.length),
                    &mut current_change,
                );
            }
        }

        touch.add_ruleoff(touch.length - 1);

        touch
    }

    pub fn inverted(&self, new_name: &str) -> Method {
        Method {
            name: new_name.to_string(),
            stage: self.stage,
            plain_lead: self.plain_lead.inverted(),
            place_notation: self.place_notation.iter().map(|x| x.reversed()).collect(),
            location_map: self.location_map.clone(),
        }
    }

    pub fn is_lead_end_variant_of(&self, other: &Method) -> bool {
        if self.lead_length() != other.lead_length() {
            return false;
        }

        let mut own_iterator = self.place_notation.iter().rev().peekable();
        let mut others_iterator = other.place_notation.iter().rev().peekable();

        // Pop the lead end PNs
        own_iterator.next();
        others_iterator.next();

        while own_iterator.peek() != None {
            if own_iterator.next() != others_iterator.next() {
                return false;
            }
        }

        true
    }
}

impl Method {
    pub fn new_with_lead_end_location(name: String, place_notation: Vec<PlaceNotation>) -> Method {
        let l = place_notation.len();

        Method::new(
            name,
            place_notation,
            hash_map! { LEAD_END_LOCATION.to_string () => l },
        )
    }

    pub fn new(
        name: String,
        place_notation: Vec<PlaceNotation>,
        location_map: HashMap<String, usize>,
    ) -> Method {
        assert!(!place_notation.is_empty());

        Method {
            name,
            stage: place_notation[0].stage,
            plain_lead: Touch::from(&place_notation[..]),
            place_notation,
            location_map,
        }
    }

    pub fn double_symmetry_from_str(
        name: &str,
        first_quarter_place_notation: &str,
        lead_end_notation: &str,
        stage: Stage,
    ) -> Method {
        Method::double_symmetry(
            name,
            &PlaceNotation::from_multiple_string(first_quarter_place_notation, stage),
            PlaceNotation::from_string(lead_end_notation, stage),
        )
    }

    pub fn double_symmetry(
        name: &str,
        first_quarter_place_notation: &[PlaceNotation],
        lead_end_notation: PlaceNotation,
    ) -> Method {
        let mut all_pns: Vec<PlaceNotation> =
            Vec::with_capacity(first_quarter_place_notation.len() * 4);

        all_pns.extend(first_quarter_place_notation);
        all_pns.extend(
            first_quarter_place_notation
                .iter()
                .rev()
                .skip(1)
                .map(|x| x.reversed()),
        );

        all_pns.push(lead_end_notation.reversed());

        all_pns.extend(
            first_quarter_place_notation
                .iter()
                .rev()
                .skip(1)
                .map(|x| x.reversed())
                .rev(),
        );
        all_pns.extend(first_quarter_place_notation.iter().rev());

        all_pns.push(lead_end_notation);

        let l = all_pns.len();

        Method::new(
            name.to_string(),
            all_pns,
            hash_map! {
                HALF_LEAD_LOCATION.to_string () => l / 2,
                LEAD_END_LOCATION.to_string () => l
            },
        )
    }

    pub fn partial_from_str(
        name: &str,
        place_notation: &str,
        lead_head: &str,
        lead_end_notation: &str,
    ) -> Method {
        let lh = Change::from(lead_head);
        let stage = lh.stage();

        Method::partial(
            name,
            &PlaceNotation::from_multiple_string(place_notation, stage),
            lh,
            PlaceNotation::from_string(lead_end_notation, stage),
        )
    }

    pub fn partial(
        name: &str,
        place_notations: &[PlaceNotation],
        lead_head: Change,
        lead_end_notation: PlaceNotation,
    ) -> Method {
        let stage = lead_head.stage();
        let lead_end = lead_head.multiply_iterator(lead_end_notation.iter());

        let mut changes = Vec::with_capacity(place_notations.len() * 2);

        let mut acc = ChangeAccumulator::new(stage);

        for pn in place_notations {
            changes.push(acc.total().clone());
            acc.accumulate_iterator(pn.iter());
        }

        changes.push(acc.total().clone());

        for i in (0..changes.len()).rev() {
            changes.push(lead_end.multiply(&changes[i]));
        }

        Method {
            name: name.to_string(),
            stage,
            plain_lead: Touch::from_changes(&changes, lead_head),
            place_notation: Vec::with_capacity(0),
            location_map: hash_map! { LEAD_END_LOCATION.to_string () => changes.len () },
        }
    }

    pub fn from_str(name: &str, place_notation_str: &str, stage: Stage) -> Method {
        let pns = PlaceNotation::from_multiple_string(place_notation_str, stage);

        let l = pns.len();

        Method::new(
            name.to_string(),
            pns,
            hash_map! { LEAD_END_LOCATION.to_string () => l },
        )
    }
}

#[cfg(test)]
mod call_tests {
    use crate::{Call, PlaceNotation, Stage, LEAD_END_LOCATION};

    #[test]
    #[should_panic]
    fn different_pn_stages() {
        Call::new(
            '-',
            vec![
                PlaceNotation::from_string("14", Stage::MAJOR),
                PlaceNotation::from_string("x", Stage::MINOR),
            ],
            LEAD_END_LOCATION,
        );
    }

    #[test]
    #[should_panic]
    fn empty_pn() {
        Call::lead_end_call_from_place_notation_string('-', "", Stage::MAJOR);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Call, Change, Method, PlaceNotation, Stage, HALF_LEAD_LOCATION};

    #[test]
    fn lead_lengths() {
        assert_eq!(
            Method::from_str("Plain Bob Triples", "7.1.7.1.7.1.7,127", Stage::TRIPLES)
                .lead_length(),
            14
        );

        assert_eq!(
            Method::from_str(
                "Cambridge Surprise Maximus",
                "x3Tx14x125Tx36x147Tx58x169Tx70x18x9Tx10xET,12",
                Stage::MAXIMUS
            )
            .lead_length(),
            48
        );
    }

    #[test]
    fn lead_fragment_generation() {
        let mut method = Method::from_str(
            "Bristol Surprise Major",
            "x58x14.58x58.36.14x14.58x14x18,18",
            Stage::MAJOR,
        );

        method.add_location("HL", 16);

        let call = Call::from_place_notation_string('h', "58", Stage::MAJOR, HALF_LEAD_LOCATION);
        let le_call = Call::lead_end_call_from_place_notation_string('-', "14", Stage::MAJOR);

        assert_eq!(
            method
                .get_lead_fragment_with_calls(0, method.lead_length(), &[call, le_call])
                .leftover_change,
            Change::from("12356478")
        );

        assert_eq!(
            method
                .get_lead_fragment_with_calls(2, method.lead_length(), &[])
                .leftover_change,
            Change::from("14253678")
        );

        assert_eq!(
            method
                .get_lead_fragment_with_calls(0, method.lead_length(), &[])
                .leftover_change,
            Change::from("14263857")
        );
    }

    #[test]
    fn lead_ends() {
        for (pns, lh) in &[
            ("7.1.7.1.7.1.7,127", "1325476"), // Plain Bob Triples
            ("x3x4x25x36x47x58x69x70x8x9x0xE,2", "1537294E6T80"), // Camb S Max
            (
                "3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7",
                "6432571",
            ), // Scientific Triples
            ("3,1.9.1.5.1", "162483957"),     // Little Grandsire Caters
        ] {
            assert_eq!(
                Method::from_str("No Name", pns, Stage::from(lh.len())).lead_end(),
                Change::from(*lh)
            );
        }
    }

    #[test]
    fn lead_end_variant() {
        for (a, b, stage_a, stage_b, exp) in &[
            (
                "1.5.1.3.2",
                "1.5.1.3.1",
                Stage::DOUBLES,
                Stage::DOUBLES,
                true,
            ),
            (
                "1.5.1.3.2",
                "1.5.1.3.1",
                Stage::DOUBLES,
                Stage::MINOR,
                false,
            ),
            (
                "x30x14x50x16x1270x38x14x50x16x90,12",
                "x30x14x50x16x1270x38x14x50x16x90,12",
                Stage::ROYAL,
                Stage::ROYAL,
                true,
            ),
            ("x1x1x1,2", "x1x1x1,1", Stage::MINOR, Stage::MINOR, true),
        ] {
            assert_eq!(
                Method::from_str("A", a, *stage_a)
                    .is_lead_end_variant_of(&Method::from_str("B", b, *stage_b)),
                *exp
            );
        }
    }

    #[test]
    fn lead_head_after_call() {
        assert_eq!(
            Method::from_str("No Name", "7.1.7.1.7.1.7,127", Stage::TRIPLES).lead_head_after_call(
                &Call::lead_end_call_from_place_notation_string('-', "147", Stage::TRIPLES)
            ),
            Change::from("1235746")
        )
    }

    #[test]
    fn double_symmetry() {
        assert_eq! (
            Method::double_symmetry (
                "Double Norwich Court Bob Major",
                &PlaceNotation::from_multiple_string ("x14x36", Stage::MAJOR),
                PlaceNotation::from_string ("18", Stage::MAJOR)
            ).plain_lead.to_string (),
            "12345678\n21436587\n24135678\n42316587\n24361578\n42635187\n24365817\n42638571\n46283751
64827315\n46287135\n64821753\n46812735\n64187253\n61482735\n16847253"
        );

        assert_eq!(
            Method::double_symmetry_from_str(
                "Bristol Surprise Maximus",
                "x5Tx14.5Tx5T.36.14x7T.58",
                "1T",
                Stage::MAXIMUS
            )
            .plain_lead
            .to_string(),
            "1234567890ET\n2143658709TE\n123468507T9E\n21438605T7E9\n241368507T9E\n4231658709TE
2413567890ET\n423157698E0T\n24351796E8T0\n234571698E0T\n32541796E8T0\n2345719E6T80\n3254791ET608
352749E16T80\n5372941ET608\n352749E1T068\n537294ET1086\n57392E4T0168\n7593E2T41086\n795E3T240168
97E5T3420618\n795E3T246081\n7593E2T40618\n57392E4T6081\n537294E6T801\n3527496E8T10\n3254769ET801
234567E98T10\n3254769E81T0\n3527496E180T\n537294E681T0\n57392E46180T\n7593E24168T0\n57392E146T80
7593E241T608\n795E32146T80\n97E53124T608\n795E132T4068\n97E531T20486\n9E75132T4068\nE97153T20486
9E175T302846\nE971T5038264\n9E17T0583624\n91E70T856342\n197ET0583624\n91E7T5038264\n197E5T302846"
        );
    }

    #[test]
    fn partial() {
        assert_eq!(
            Method::partial_from_str("Partial Method", "x30", "1352749608", "12")
                .plain_lead
                .to_string(),
            "1234567890\n2143658709\n1246385079\n1357294068\n3152749608\n1325476980"
        );

        assert_eq!(
            Method::partial(
                "Partial Method",
                &PlaceNotation::from_multiple_string("x30", Stage::ROYAL),
                Change::from("1352749608"),
                PlaceNotation::from_string("12", Stage::ROYAL)
            )
            .plain_lead
            .to_string(),
            "1234567890\n2143658709\n1246385079\n1357294068\n3152749608\n1325476980"
        );
    }

    #[test]
    fn lead_heads() {
        for (pns, lh) in &[
            ("7.1.7.1.7.1.7,127", "1352746"), // Plain Bob Triples
            ("x3x4x25x36x47x58x69x70x8x9x0xE,2", "157392E4T608"), // Camb S Max
            (
                "3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7",
                "4623751",
            ), // Scientific Triples
            ("3,1.9.1.5.1", "126849375"),     // Little Grandsire Caters
        ] {
            assert_eq!(
                *Method::from_str("No Name", pns, Stage::from(lh.len())).lead_head(),
                Change::from(*lh)
            );
        }
    }

    #[test]
    fn inversion() {
        for (pns, lh) in &[
            ("7.1.7.1.7.1.7,127", "2416357"), // Plain Bob Triples
            ("x3x4x25x36x47x58x69x70x8x9x0xE,2", "537192E4068T"), // Camb S Max
            (
                "3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7",
                "7315624",
            ), // Scientific Triples
        ] {
            assert_eq!(
                *Method::from_str("No Name", pns, Stage::from(lh.len()))
                    .inverted("Enam On")
                    .lead_head(),
                Change::from(*lh)
            );
        }
    }
}
