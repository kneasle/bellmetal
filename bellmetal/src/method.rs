use crate::types::Stage;

pub struct Method<'a> {
    pub name : &'a str,
    pub stage : Stage
}
