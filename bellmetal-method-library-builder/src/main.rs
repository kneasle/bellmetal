use std::fs::File;
use std::io::{self, prelude::*, BufRead};
use std::path::Path;

use bellmetal::*;

fn main() {
    let output_path = concat!(env!("PWD"), "/../CC_library.txt");
    let input_path = concat!(env!("PWD"), "/CCCBR_methods.txt");

    // Consume the method library
    let mut count = 0;
    let mut num_methods = 0;
    let mut method_library_string = String::with_capacity(10_000_000);
    let mut name_buffer = String::with_capacity(100);
    let mut place_notations: Vec<PlaceNotation> = Vec::with_capacity(1_000);

    if let Ok(lines) = read_lines(input_path) {
        for line in lines {
            if let Ok(l) = line {
                count += 1;

                if count > 5 {
                    let parts = l.split("\t");

                    let mut stage = Stage::ZERO;

                    name_buffer.clear();
                    place_notations.clear();

                    for (i, part) in parts.enumerate() {
                        if i == 1 {
                            name_buffer.push_str(&part);
                        }

                        if i == 5 {
                            stage = Stage::from(part.parse::<usize>().unwrap());
                        }

                        if i >= 10 {
                            assert!(stage != Stage::ZERO);

                            place_notations.push(PlaceNotation::from_string(&part, stage));
                        }
                    }

                    serialise_method(
                        &Method::new_with_lead_end_location(name_buffer.clone(), place_notations.clone()),
                        &mut method_library_string,
                    );

                    method_library_string.push('\n');

                    num_methods += 1;
                }

                if num_methods == 100 {
                    // break;
                }
            }
        }
    }

    method_library_string.pop(); // remove unnecessary newline

    let mut out_file = File::create(output_path).unwrap();
    out_file.write(method_library_string.as_bytes()).unwrap();
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
