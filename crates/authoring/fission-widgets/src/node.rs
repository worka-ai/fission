use crate::{button::Button, row::Row, text::Text, Desugar, LoweringContext};
use fission_ir::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Text(Text),
    Row(Row),
    Button(Button),
}

impl Desugar for Node {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        match self {
            Node::Text(widget) => widget.desugar(cx),
            Node::Row(widget) => widget.desugar(cx),
            Node::Button(widget) => widget.desugar(cx),
        }
    }
}

impl From<Text> for Node {
    fn from(widget: Text) -> Self {
        Node::Text(widget)
    }
}

impl From<Row> for Node {
    fn from(widget: Row) -> Self {
        Node::Row(widget)
    }
}

impl From<Button> for Node {
    fn from(widget: Button) -> Self {
        Node::Button(widget)
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::Text(Text::default())
    }
}
