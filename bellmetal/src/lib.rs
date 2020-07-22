#![allow(dead_code)]

pub mod change;
pub mod consts;
pub mod coursing_order;
pub mod method;
pub mod method_library;
pub mod music_scoring;
pub mod place_notation;
pub mod proving;
pub mod touch;
pub mod touch_generation;
pub mod touch_iterator;
pub mod transposition;
pub mod types;
pub mod utils;

// Flatten the module structure for easier importing
pub use change::{Change, ChangeAccumulator, ChangeCollectIter};
pub use consts::{is_bell_name, name_to_number, BELL_NAMES, MAX_STAGE};
pub use coursing_order::{
    first_plain_bob_lead_head, plain_bob_lead_head, BasicCoursingOrderIterator, CoursingOrder,
    CoursingOrderIterator, LeadheadCoursingOrderIterator, PlainCoursingOrderIterator, RunSection,
};
pub use method::{Call, Method, HALF_LEAD_LOCATION, LEAD_END_LOCATION};
pub use method_library::{deserialise_method, serialise_method, MethodLibrary};
pub use music_scoring::{DefaultScoring, MusicScoring};
pub use place_notation::PlaceNotation;
pub use proving::{
    canon_copy, canon_fixed_treble_cyclic, canon_full_cyclic, CompactHashProver,
    FullProvingContext, HashProver, NaiveProver, ProvingContext,
};
pub use touch::{BasicTouchIterator, Row, Touch};
pub use touch_generation::{one_part_spliced_touch, single_method_touch};
pub use touch_iterator::{MultiChainTouchIterator, TouchIterator, TransfiguredTouchIterator};
pub use transposition::{MultiplicationIterator, Transposition};
pub use types::{Bell, Mask, MaskMethods, Number, Parity, Place, Stage, Stroke};
pub use utils::{closure, extent};
