pub mod node_id;
pub mod op;

pub use node_id::NodeId;
pub use op::{Op, StructuralOp, LayoutOp, PaintOp};

pub const IR_VERSION: u32 = 1;
