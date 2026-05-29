use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cart {
    pub items: Vec<String>,
}

pub trait CartService: Send + Sync {
    fn load(&self, session_id: &str) -> Cart;
    fn add_item(&self, session_id: &str, slug: &str) -> Cart;
    fn clear(&self, session_id: &str) -> Cart;
}

#[derive(Default)]
pub struct InMemoryCartService {
    carts: RwLock<BTreeMap<String, Cart>>,
}

impl CartService for InMemoryCartService {
    fn load(&self, session_id: &str) -> Cart {
        self.carts
            .read()
            .expect("cart service lock poisoned")
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    fn add_item(&self, session_id: &str, slug: &str) -> Cart {
        let mut carts = self.carts.write().expect("cart service lock poisoned");
        let cart = carts.entry(session_id.to_string()).or_default();
        cart.items.push(slug.to_string());
        cart.clone()
    }

    fn clear(&self, session_id: &str) -> Cart {
        self.carts
            .write()
            .expect("cart service lock poisoned")
            .remove(session_id)
            .unwrap_or_default()
    }
}

impl InMemoryCartService {
    pub fn clear_all(&self) {
        self.carts
            .write()
            .expect("cart service lock poisoned")
            .clear();
    }
}

static CART_SERVICE: OnceLock<InMemoryCartService> = OnceLock::new();

pub fn cart_service() -> &'static InMemoryCartService {
    CART_SERVICE.get_or_init(InMemoryCartService::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_cart_is_keyed_by_session() {
        let service = InMemoryCartService::default();
        service.add_item("a", "charizard-holo");
        service.add_item("b", "pikachu-yellow-cheeks");

        assert_eq!(service.load("a").items, vec!["charizard-holo"]);
        assert_eq!(service.load("b").items, vec!["pikachu-yellow-cheeks"]);
        assert!(service.load("missing").items.is_empty());
    }
}
