use crate::{Method, PlaceNotation, Stage};

use std::fs;
use std::path::Path;

const DELIMITER: char = '|';

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct StoredMethod {
    name: String,
    place_notation: Vec<PlaceNotation>,
    stage: Stage,
}

impl StoredMethod {
    pub fn to_method(&self) -> Method {
        Method::new_with_lead_end_location(self.name.clone(), self.place_notation.clone())
    }
}

impl StoredMethod {
    pub fn new(name: String, place_notation: Vec<PlaceNotation>, stage: Stage) -> StoredMethod {
        StoredMethod {
            name,
            place_notation,
            stage,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct MethodLibrary {
    stored_methods: Vec<StoredMethod>,
}

impl MethodLibrary {
    pub fn all_methods<'a>(&'a self) -> impl Iterator<Item = Method> + 'a {
        self.stored_methods.iter().map(|x| x.to_method())
    }

    pub fn get_method_by_notation(&self, place_notations: &[PlaceNotation]) -> Option<Method> {
        for m in &self.stored_methods {
            if m.place_notation == place_notations {
                return Some(m.to_method());
            }
        }

        None
    }

    pub fn get_method(&self, string: &str) -> Option<Method> {
        for stored_method in &self.stored_methods {
            if stored_method.name == string {
                return Some(stored_method.to_method());
            }
        }

        None
    }
}

impl MethodLibrary {
    pub fn from_string(string: &str) -> MethodLibrary {
        MethodLibrary::from_string_filtered(string, None)
    }

    pub fn from_string_filtered(string: &str, stage: Option<Stage>) -> MethodLibrary {
        let mut stored_methods: Vec<StoredMethod> = Vec::with_capacity(2000);

        match stage {
            None => {
                for l in string.lines() {
                    stored_methods.push(deserialise_stored_method(l));
                }
            }
            Some(_) => {
                for l in string.lines() {
                    if let Some(m) = deserialise_stored_method_filtered(l, stage) {
                        stored_methods.push(m);
                    }
                }
            }
        }

        MethodLibrary { stored_methods }
    }

    pub fn from_file(path: &Path) -> MethodLibrary {
        MethodLibrary::from_string(&fs::read_to_string(&path).expect("Couldn't read file"))
    }

    pub fn from_file_filtered(path: &Path, stage: Option<Stage>) -> MethodLibrary {
        MethodLibrary::from_string_filtered(
            &fs::read_to_string(&path).expect("Couldn't read file"),
            stage,
        )
    }
}

pub fn deserialise_method(string: &str) -> Method {
    deserialise_stored_method(string).to_method()
}

fn deserialise_stored_method_filtered(
    string: &str,
    stage_filter: Option<Stage>,
) -> Option<StoredMethod> {
    let mut parts = string.split(DELIMITER);

    let stage = Stage::from(parts.next().unwrap().parse::<usize>().unwrap());
    if let Some(s) = stage_filter {
        if stage != s {
            return None;
        }
    }
    let name = parts.next().unwrap().to_string();
    let place_notation = PlaceNotation::from_multiple_string(parts.next().unwrap(), stage);

    Some(StoredMethod::new(name, place_notation, stage))
}

fn deserialise_stored_method(string: &str) -> StoredMethod {
    let mut parts = string.split(DELIMITER);

    let stage = Stage::from(parts.next().unwrap().parse::<usize>().unwrap());
    let name = parts.next().unwrap().to_string();
    let place_notation = PlaceNotation::from_multiple_string(parts.next().unwrap(), stage);

    StoredMethod::new(name, place_notation, stage)
}

pub fn serialise_method(method: &Method, string: &mut String) {
    string.push_str(&method.stage.as_usize().to_string());
    string.push(DELIMITER);
    string.push_str(&method.name);
    string.push(DELIMITER);
    PlaceNotation::write_notations_to_string_compact(&method.place_notations, string);
}

#[cfg(test)]
mod tests {
    use crate::{
        deserialise_method, serialise_method, Method, MethodLibrary, PlaceNotation, Stage,
    };

    #[test]
    fn serialisation() {
        let mut s = String::with_capacity(100);

        for m in &[
            Method::from_str("St Remigius Place Singles", "3.1.3,123", Stage::SINGLES),
            Method::from_str("Reverse Carter Singles", "1.1.3,3.1.1.1", Stage::SINGLES),
            Method::from_str("Plain Bob Doubles", "5.1.5.1.5,125", Stage::DOUBLES),
            Method::from_str(
                "\"Brent\" Surprise Minor",
                "3456-56.14-56-36.12-12.56,12",
                Stage::MINOR,
            ),
            Method::from_str("Zzzzz... Bob Minor", "56.14.1256.36.12.56,16", Stage::MINOR),
            Method::from_str(
                "Scientific Triples",
                "3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7",
                Stage::TRIPLES,
            ),
            Method::from_str(
                "Bristol Surprise Major",
                "-58-14.58-58.36.14-14.58-14-18,18",
                Stage::MAJOR,
            ),
            Method::from_str("Plain Bob Major", "-18-18-18-18,12", Stage::MAJOR),
            Method::from_str(
                "Cornwall Surprise Major",
                "-56-14-56-38-14-58-14-58,18",
                Stage::MAJOR,
            ),
            Method::from_str(
                "Cambridge Surprise Major",
                "-38-14-1258-36-14-58-16-78,12",
                Stage::MAJOR,
            ),
            Method::from_str(
                "Lessness Surprise Major",
                "-38-14-56-16-12-58-14-58,12",
                Stage::MAJOR,
            ),
            Method::from_str("Grandsire Caters", "3,1.9.1.9.1.9.1.9.1", Stage::CATERS),
        ] {
            s.clear();
            serialise_method(m, &mut s);

            let method = deserialise_method(&s.clone());

            assert_eq!(method.name, m.name);
            assert_eq!(method.stage, m.stage);
            assert_eq!(method.place_notations, mplace_notationsn);
        }
    }

    #[test]
    fn library() {
        let meth_string = "3|St Remigius Place Singles|3.1.3,2
3|Titanic Singles|3.1,3
4|Grandsire Minimus|3,1x1x
4|Ada Minimus|2x2.1.2,1x
6|NFFC Treble Place Minor|x5x4x2x23x45x5,2
6|Gasherbrum II Treble Place Minor|x5x4x3.2.345.2.3x34,1
7|Little Orchard Bob Triples|5.1.5.23.7.45.7,2
7|Cold Ash Bob Triples|5.1.7.3.7.5.7,25
7|Fellowship of the Ring Alliance Triples|7.1.5.3.7.5.3.5.7,1
8|Lagargawan Bob Major|567.4.2567.367.34.345.256.7,2
8|Unchhera Bob Major|567.456x236.34.5.23456.7,2
8|Ancient Society of Efquire Leeds Youths Treble Place Major|x3x4x5x6x4x5x367x7,2
8|Cockup Bridge Treble Place Major|x3x4x5x36x4x1x2567x7,2
8|Tiffield Treble Bob Major|34x34.1x2x1x2x1x2x7,2
8|Fairford Surprise Major|x36x6x5x36x2x3.4x4.7,2
12|Folgate Surprise Maximus|3x56.4x56x3x4x5x4x5x4x5x4x5,2
12|Cirencester Surprise Maximus|70.36x450.78.9x6x29x0x90x7x78x67x690x1,2
12|Ripon Surprise Maximus|3x5.4x2x6x4x3.4x4.5.4x4.5.4x4.5,2";

        let method_lib = MethodLibrary::from_string(meth_string);
        let method_lib_triples =
            MethodLibrary::from_string_filtered(meth_string, Some(Stage::MAJOR));

        assert_eq!(method_lib.all_methods().count(), 18);

        assert_eq!(
            method_lib
                .get_method_by_notation(&PlaceNotation::from_multiple_string(
                    "x5x4x2x23x45x5,2",
                    Stage::MINOR
                ))
                .unwrap()
                .name,
            "NFFC Treble Place Minor"
        );

        assert_eq!(
            method_lib_triples.get_method_by_notation(&PlaceNotation::from_multiple_string(
                "x5x4x2x23x45x5,2",
                Stage::MINOR
            )),
            None
        );

        assert_eq!(
            method_lib.get_method_by_notation(&PlaceNotation::from_multiple_string(
                "x5x4x2x23x45x3,2",
                Stage::MINOR
            )),
            None
        );

        assert_eq!(
            method_lib.get_method("Fellowship of the Ring Alliance Triples"),
            Some(Method::from_str(
                "Fellowship of the Ring Alliance Triples",
                "7.1.5.3.7.5.3.5.7,1",
                Stage::TRIPLES
            ))
        );

        assert_eq!(
            method_lib_triples.get_method("Fellowship of the Ring Alliance Triples"),
            None
        );

        assert_eq!(
            method_lib.get_method("Fellowship of The Ring Alliance Triples"),
            None
        );
    }
}
