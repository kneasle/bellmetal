#![allow(dead_code)]

pub mod change;
pub mod consts;
pub mod coursing_order;
pub mod method;
pub mod method_library;
pub mod place_notation;
pub mod proving;
pub mod touch;
pub mod touch_iterator;
pub mod touch_generation;
pub mod transposition;
pub mod types;
pub mod utils;

// Flatten the module structure for easier importing
pub use change::{ Change, ChangeAccumulator, ChangeIter };
pub use consts::{ MAX_STAGE, BELL_NAMES, is_bell_name, name_to_number };
pub use coursing_order::{ CoursingOrder, CoursingOrderIterator, BasicCoursingOrderIterator, LeadheadCoursingOrderIterator, PlainCoursingOrderIterator };
pub use method::{ Method, Call };
pub use method_library::{ MethodLibrary, serialise_method, deserialise_method };
pub use place_notation::PlaceNotation;
pub use proving::{ ProvingContext, FullProvingContext, NaiveProver, HashProver, CompactHashProver, canon_copy, canon_fixed_treble_cyclic, canon_full_cyclic };
pub use touch::{ Row, Touch, BasicTouchIterator };
pub use touch_iterator::{ TouchIterator, TransfiguredTouchIterator };
pub use transposition::{ Transposition, MultiplicationIterator };
pub use types::{ Stroke, Bell, Place, Parity, Stage, Number, Mask, MaskMethods };
pub use touch_generation::{ one_part_spliced_touch };
pub use utils::{ extent, closure };
