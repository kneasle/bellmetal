use crate::types::{ Stage, Bell };

pub struct Row<'a> {
    index : u32,
    touch : &'a Touch<'a>,
    bells : &'a [Bell]
}

struct Touch<'a> {
    stage : Stage,
    rows : &'a Vec<Row <'a>>,
    bells : &'a Vec<Bell>
}
