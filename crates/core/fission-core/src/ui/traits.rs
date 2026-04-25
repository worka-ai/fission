//! Lowering traits for converting widgets into the intermediate representation.

use crate::lowering::LoweringContext;
use fission_ir::NodeId;
use std::fmt::Debug;

/// Converts a widget struct into `fission-ir` nodes.
///
/// Every built-in widget implements `Lower`. The method receives a
/// [`LoweringContext`] and returns the root [`NodeId`] of the emitted IR
/// subgraph.
pub trait Lower {
    /// Lower this widget into the IR, returning the root node id.
    fn lower(&self, cx: &mut LoweringContext) -> NodeId;
}

/// Object-safe variant of [`Lower`] for use inside [`CustomNode`](crate::ui::CustomNode).
///
/// Implement this trait when you need custom lowering logic that cannot be
/// expressed with the built-in widget primitives.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Debug)]
/// struct MyCanvasLowerer;
///
/// impl LowerDyn for MyCanvasLowerer {
///     fn lower_dyn(&self, cx: &mut LoweringContext) -> NodeId {
///         // emit custom IR nodes...
///         cx.next_node_id()
///     }
///
///     fn stable_key(&self) -> u64 {
///         0xCAFE
///     }
/// }
/// ```
pub trait LowerDyn: Send + Sync + Debug {
    /// Lower this widget into the IR, returning the root node id.
    fn lower_dyn(&self, cx: &mut LoweringContext) -> NodeId;
    /// A stable key used for structural diffing. Override to provide a
    /// content-based hash.
    fn stable_key(&self) -> u64 {
        0
    }
}
