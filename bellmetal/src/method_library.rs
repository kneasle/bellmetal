use crate::{ Method, PlaceNotation, Stage };

use std::path::Path;
use std::fs;

const DELIMITER : char = '|';

pub struct MethodLibrary {
    methods : Vec<Method>
}

impl MethodLibrary {
    pub fn get_method (&self, string : &str) -> Option<&Method> {
        for m in &self.methods {
            if m.name == string {
                return Some (m);
            }
        }

        None
    }
}

impl MethodLibrary {
    pub fn from_string (string : &String) -> MethodLibrary {
        let mut methods : Vec<Method> = Vec::with_capacity (2000);

        for s in string.lines () {
            methods.push (deserialise_method (s));
        }

        MethodLibrary {
            methods : methods
        }
    }

    pub fn from_file (path : &Path) -> MethodLibrary {
        MethodLibrary::from_string (&fs::read_to_string (&path).expect ("Couldn't read file"))
    }
}

pub fn deserialise_method (string : &str) -> Method {
    let mut parts =  string.split (DELIMITER);

    let name = parts.next ().unwrap ().to_string ();
    let stage = Stage::from (parts.next ().unwrap ().parse::<usize> ().unwrap ());
    let place_notation = PlaceNotation::from_multiple_string (parts.next ().unwrap (), stage);

    Method::new (name, place_notation)
}

pub fn serialise_method (method : &Method, string : &mut String) {
    string.push_str (&method.name);
    string.push (DELIMITER);
    string.push_str (&method.stage.as_usize ().to_string ());
    string.push (DELIMITER);
    PlaceNotation::into_multiple_string_short (&method.place_notation, string);
}

#[cfg(test)]
mod lib_tests {
    use crate::{ Method, Stage, deserialise_method, serialise_method };

    #[test]
    fn to_from_text () {
        let mut s = String::with_capacity (100);

        for m in &[
            Method::from_str ("St Remigius Place Singles", "3.1.3,123", Stage::SINGLES),
            Method::from_str ("Plain Bob Doubles", "5.1.5.1.5,125", Stage::DOUBLES),
            Method::from_str ("\"Brent\" Surprise Minor", "3456-56.14-56-36.12-12.56,12", Stage::MINOR),
            Method::from_str ("Zzzzz... Bob Minor", "56.14.1256.36.12.56,16", Stage::MINOR),
            Method::from_str ("Bristol Surprise Major", "-58-14.58-58.36.14-14.58-14-18,18", Stage::MAJOR),
            Method::from_str ("Plain Bob Major", "-18-18-18-18,12", Stage::MAJOR),
            Method::from_str ("Cornwall Surprise Major", "-56-14-56-38-14-58-14-58,18", Stage::MAJOR),
            Method::from_str ("Cambridge Surprise Major", "-38-14-1258-36-14-58-16-78,12", Stage::MAJOR),
            Method::from_str ("Lessness Surprise Major", "-38-14-56-16-12-58-14-58,12", Stage::MAJOR)
        ] {
            s.clear ();
            serialise_method (m, &mut s);

            let method = deserialise_method (&s.clone ());

            assert_eq! (method.name, m.name);
            assert_eq! (method.stage, m.stage);
            assert_eq! (method.place_notation, m.place_notation);
        }
    }
}
