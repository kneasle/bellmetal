#![allow(dead_code)]

pub mod change;
pub mod consts;
pub mod method;
pub mod method_library;
pub mod place_notation;
pub mod proving;
pub mod touch;
pub mod touch_generation;
pub mod transposition;
pub mod types;
pub mod utils;

// Flatten the module structure for easier importing
pub use change::{ Change, ChangeAccumulator };
pub use consts::{ MAX_STAGE, BELL_NAMES, is_bell_name, name_to_number };
pub use method::{ Method, Call };
pub use method_library::{ MethodLibrary, serialise_method, deserialise_method };
pub use place_notation::PlaceNotation;
pub use proving::{ ProvingContext, NaiveProver, HashProver };
pub use touch::{ Row, Touch, BasicTouchIterator, TransfiguredTouchIterator, ConcatTouchIterator, AppendedTouchIterator, TouchIterator };
pub use transposition::{ Transposition, TranspositionIterator, MultiplicationIterator };
pub use types::{ Bell, Place, Parity, Stage, Number, Mask, MaskMethods };
pub use touch_generation::{ one_part_spliced_touch };
