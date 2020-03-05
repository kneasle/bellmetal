#![allow(dead_code)]

pub mod change;
pub mod consts;
pub mod method;
pub mod place_notation;
pub mod touch;
pub mod touch_generation;
pub mod transposition;
pub mod types;
pub mod utils;

// Flatten the module structure for easier importing
pub use change::{ Change, ChangeAccumulator };
pub use consts::{ MAX_STAGE, BELL_NAMES, is_bell_name, name_to_number };
pub use method::{ Method, Call };
pub use place_notation::PlaceNotation;
pub use touch::{ Row, Touch, BasicTouchIterator, TransfiguredTouchIterator, ConcatTouchIterator, TouchIterator };
pub use transposition::{ Transposition, TranspositionIterator, MultiplicationIterator };
pub use types::{ Bell, Place, Parity, Stage, Number, Mask, MaskMethods };
pub use touch_generation::{ one_part_spliced_touch };
