use dioxus_core::{DynamicNode::Component, VNode};

/// Extension trait for [`VNode`](dioxus_core::VNode).
pub trait VNodeExt {
    /// Returns `true` if `self` has a given component node as its child.
    fn has_component(&self, name: &str) -> bool;
}

impl<'a> VNodeExt for VNode<'a> {
    fn has_component(&self, name: &str) -> bool {
        self.dynamic_nodes.iter().any(|node| {
            if let Component(node) = node {
                node.name == name
            } else {
                false
            }
        })
    }
}
