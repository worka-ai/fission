use fission_macros::Action;
use fission_core::{ActionId, Action};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MyTestAppAction { pub value: u32 }

#[test]
fn test_derive_action_id_stability() {
    let action1 = MyTestAppAction { value: 1 };
    let action2 = MyTestAppAction { value: 2 };

    // ActionId should be stable and identical for the same type
    assert_eq!(action1.id(), action2.id());

    // Verify the generated ID matches expectation.
    // The macro generates ID based on module path.
    // In integration tests, the module path is the test file module.
    let expected_id = ActionId::from_name("derive_action::MyTestAppAction");
    assert_eq!(action1.id(), expected_id);
}

#[test]
fn test_derive_action_serialization() {
    let action = MyTestAppAction { value: 42 };
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: MyTestAppAction = serde_json::from_str(&serialized).unwrap();

    assert_eq!(action, deserialized);
}
