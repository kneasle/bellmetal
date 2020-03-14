use crate::{ Touch, Row };

pub trait ProvingContext {
    fn prove (&mut self, touch : &Touch) -> bool;
}






pub struct NaiveProver { }

impl ProvingContext for NaiveProver {
    fn prove (&mut self, touch : &Touch) -> bool {
        let mut rows : Vec<Row> = touch.row_iterator ().collect ();

        rows.sort ();

        for i in 1..rows.len () {
            if rows [i - 1] == rows [i] {
                return false;
            }
        }

        true
    }
}
