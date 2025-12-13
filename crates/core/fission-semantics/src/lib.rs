use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Button,
    Text,
    Image,
    Checkbox,
    Slider,
    Input,
    List,
    ListItem,
    // Future roles...
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Semantics {
    pub role: Role,
    pub label: Option<String>,
    pub value: Option<String>, // Structured value could be more complex, String for MVP
    pub actions: ActionSet,
    pub focusable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ActionSet {
    // List of action IDs supported by this node.
    // For now, we use strings or u128s as placeholders for ActionId
    pub supported: Vec<u128>,
}
