#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeValue<T> {
    Principal { value: T },
    All { value: T },
    Cut { value: T },
}

impl<T: Default> Default for NodeValue<T> {
    fn default() -> Self {
        Self::Principal {
            value: T::default(),
        }
    }
}

impl<T> NodeValue<T> {
    pub fn pv_node(value: T) -> Self {
        Self::Principal { value }
    }

    pub fn all_node(value: T) -> Self {
        Self::All { value }
    }

    pub fn cut_node(value: T) -> Self {
        Self::Cut { value }
    }
}
