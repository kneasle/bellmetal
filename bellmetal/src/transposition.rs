use crate::types::*;

pub trait Transposition {
    fn stage (&self) -> Stage;
    fn bell_at (&self, place : Place) -> Bell;
}
