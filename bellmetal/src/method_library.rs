use crate::{ Method, PlaceNotation, Stage };

use std::path::Path;
use std::fs;

const DELIMITER : char = '|';

struct StoredMethod {
    name : String,
    place_notation : Vec<PlaceNotation>,
    stage : Stage
}

impl StoredMethod {
    pub fn to_method (&self) -> Method {
        Method::new (self.name.clone (), self.place_notation.clone ())
    }
}

impl StoredMethod {
    pub fn new (name : String, place_notation : Vec<PlaceNotation>, stage : Stage) -> StoredMethod {
        StoredMethod {
            name : name,
            place_notation : place_notation,
            stage : stage
        }
    }
}






pub struct MethodLibrary {
    stored_methods : Vec<StoredMethod>,
}

impl MethodLibrary {
    pub fn get_method (&mut self, string : &str) -> Option<Method> {
        for stored_method in &self.stored_methods {
            if stored_method.name == string {
                return Some (stored_method.to_method ());
            }
        }

        None
    }
}

impl MethodLibrary {
    pub fn from_string (string : &String) -> MethodLibrary {
        let mut stored_methods : Vec<StoredMethod> = Vec::with_capacity (2000);

        for s in string.lines () {
            stored_methods.push (deserialise_stored_method (s));
        }

        MethodLibrary {
            stored_methods : stored_methods,
        }
    }

    pub fn from_string_filtered (string : &String, stage : Option<Stage>) -> MethodLibrary {
        let mut stored_methods : Vec<StoredMethod> = Vec::with_capacity (2000);
        
        match stage {
            None => {
                for l in string.lines () {
                    stored_methods.push (deserialise_stored_method (l));
                }
            }
            Some (_) => {
                for l in string.lines () {
                    match deserialise_stored_method_filtered (l, stage) {
                        Some (m) => {
                            stored_methods.push (m);
                        }
                        None => { }
                    }
                }
            }
        }

        MethodLibrary {
            stored_methods : stored_methods,
        }
    }

    pub fn from_file (path : &Path) -> MethodLibrary {
        MethodLibrary::from_string (&fs::read_to_string (&path).expect ("Couldn't read file"))
    }

    pub fn from_file_filtered (path : &Path, stage : Option<Stage>) -> MethodLibrary {
        MethodLibrary::from_string_filtered (&fs::read_to_string (&path).expect ("Couldn't read file"), stage)
    }
}






pub fn deserialise_method (string : &str) -> Method {
    deserialise_stored_method (string).to_method ()
}

fn deserialise_stored_method_filtered (string : &str, stage_filter : Option<Stage>) -> Option<StoredMethod> {
    let mut parts =  string.split (DELIMITER);

    let stage = Stage::from (parts.next ().unwrap ().parse::<usize> ().unwrap ());
    match stage_filter {
        Some (s) => {
            if stage != s {
                return None;
            }
        }
        _ => { }
    }
    let name = parts.next ().unwrap ().to_string ();
    let place_notation = PlaceNotation::from_multiple_string (parts.next ().unwrap (), stage);

    Some (StoredMethod::new (name, place_notation, stage))
}

fn deserialise_stored_method (string : &str) -> StoredMethod {
    let mut parts =  string.split (DELIMITER);

    let stage = Stage::from (parts.next ().unwrap ().parse::<usize> ().unwrap ());
    let name = parts.next ().unwrap ().to_string ();
    let place_notation = PlaceNotation::from_multiple_string (parts.next ().unwrap (), stage);

    StoredMethod::new (name, place_notation, stage)
}

pub fn serialise_method (method : &Method, string : &mut String) {
    string.push_str (&method.stage.as_usize ().to_string ());
    string.push (DELIMITER);
    string.push_str (&method.name);
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
            Method::from_str ("Reverse Carter Singles", "1.1.3,3.1.1.1", Stage::SINGLES),
            Method::from_str ("Plain Bob Doubles", "5.1.5.1.5,125", Stage::DOUBLES),
            Method::from_str ("\"Brent\" Surprise Minor", "3456-56.14-56-36.12-12.56,12", Stage::MINOR),
            Method::from_str ("Zzzzz... Bob Minor", "56.14.1256.36.12.56,16", Stage::MINOR),
            Method::from_str ("Scientific Triples", "3.1.7.1.5.1.7.1.7.5.1.7.1.7.1.7.1.7.1.5.1.5.1.7.1.7.1.7.1.7", Stage::TRIPLES),
            Method::from_str ("Bristol Surprise Major", "-58-14.58-58.36.14-14.58-14-18,18", Stage::MAJOR),
            Method::from_str ("Plain Bob Major", "-18-18-18-18,12", Stage::MAJOR),
            Method::from_str ("Cornwall Surprise Major", "-56-14-56-38-14-58-14-58,18", Stage::MAJOR),
            Method::from_str ("Cambridge Surprise Major", "-38-14-1258-36-14-58-16-78,12", Stage::MAJOR),
            Method::from_str ("Lessness Surprise Major", "-38-14-56-16-12-58-14-58,12", Stage::MAJOR),
            Method::from_str ("Grandsire Caters", "3,1.9.1.9.1.9.1.9.1", Stage::CATERS)
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
