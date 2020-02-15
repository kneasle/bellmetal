#![allow(dead_code)]

pub mod change;
pub mod consts;
pub mod method;
pub mod place_notation;
pub mod touch;
pub mod transposition;
pub mod types;
pub mod utils;

// Flatten the module structure for easier importing
pub use change::{ Change, ChangeAccumulator };
pub use consts::{ MAX_STAGE, BELL_NAMES, is_bell_name, name_to_number };
pub use method::{ Method, Call };
pub use place_notation::PlaceNotation;
pub use touch::{ Row, Touch, TouchIterator };
pub use transposition::{ Transposition };
pub use types::{ Bell, Place, Stage, Number, Mask, MaskMethods };
