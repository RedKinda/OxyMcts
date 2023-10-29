/*
Tree implementation that is
a) based on dashmap
b) node id is hash of state (generic T)
c) acyclic
d) select new root and prune unreachable nodes
*/

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use tracing::{debug, trace};

use crate::Num;

pub type NodeId = u64;
pub trait Hashed {
    fn hash(&self) -> NodeId;
}

#[derive(Clone)]
pub struct Tree<T> {
    map: DashMap<NodeId, Node<T>>,
    root: NodeId,
}

#[derive(Clone)]
struct Node<T> {
    parent: NodeId,
    children: Vec<NodeId>,
    value: T,
}

pub struct NodeRef<'a, T> {
    tree: &'a Tree<T>,
    node: Ref<'a, u64, Node<T>>,
}

pub struct NodeMutRef<'a, T> {
    tree: &'a Tree<T>,
    node: RefMut<'a, u64, Node<T>>,
}

impl<T: Hashed> Tree<T> {
    pub fn new(root: T) -> Self {
        let root_id = root.hash();

        let mut map = DashMap::new();
        map.insert(
            root_id,
            Node {
                parent: 0,
                children: vec![],
                value: root,
            },
        );
        Self { map, root: root_id }
    }

    pub fn with_capacity(root: T, capacity: usize) -> Self {
        let root_id = root.hash();

        let mut map = DashMap::with_capacity(capacity);
        map.insert(
            root_id,
            Node {
                parent: 0,
                children: vec![],
                value: root,
            },
        );
        Self { map, root: root_id }
    }

    pub fn root(&self) -> NodeMutRef<'_, T> {
        let node = self.map.get_mut(&self.root).unwrap();
        NodeMutRef { tree: self, node }
    }

    pub fn root_id(&self) -> NodeId {
        self.root
    }

    pub fn get_mut(&self, id: u64) -> Option<NodeMutRef<'_, T>> {
        trace!("get MUT {}", id);
        let node = self.map.get_mut(&id)?;
        Some(NodeMutRef { tree: self, node })
    }
    pub fn get(&self, id: u64) -> Option<NodeRef<'_, T>> {
        trace!("get {}", id);
        let node = self.map.get(&id)?;
        Some(NodeRef { tree: self, node })
    }
}

impl<'a, T: Hashed> NodeMutRef<'a, T> {
    pub fn id(&self) -> NodeId {
        self.node.key().clone()
    }
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.node.value_mut().value
    }
    pub fn value(&self) -> &T {
        &self.node.value().value
    }

    pub fn add_child(mut self, value: T) -> NodeMutRef<'a, T> {
        trace!("add_child {} to {}", value.hash(), self.node.key());
        let id = value.hash();
        self.node.children.push(id);
        let tree = self.tree;
        let parent_id = self.node.key().clone();
        drop(self);

        let res = tree.map.entry(id).or_insert_with(|| Node {
            parent: parent_id,
            children: vec![],
            value,
        });
        NodeMutRef { tree, node: res }
    }
    pub fn parent_id(self) -> NodeId {
        if self.node.parent == 0 {
            panic!("root has no parent")
        } else {
            self.node.parent
        }
    }
}

impl<T: Hashed> NodeRef<'_, T> {
    pub fn id(&self) -> NodeId {
        self.node.key().clone()
    }
    pub fn has_children(&self) -> bool {
        !self.node.children.is_empty()
    }
    pub fn value(&self) -> &T {
        &self.node.value().value
    }

    // takes a function that returns an u64
    pub fn get_best_child(self, f: impl Fn(&T) -> Num) -> Option<NodeId> {
        let mut max_score = None;
        let mut best_child = None;
        let children_ids = self.node.children.clone();
        let tree = self.tree;
        drop(self);

        for child_id in children_ids {
            let child = tree.map.get(&child_id).unwrap();
            let score = f(&child.value().value);
            if max_score.is_none() || score > max_score.unwrap() {
                max_score = Some(score);
                best_child = Some(child_id.clone());
            }
        }

        best_child
    }
}

impl<T> Drop for NodeRef<'_, T> {
    fn drop(&mut self) {
        trace!("drop {}", self.node.key());
    }
}

impl<T> Drop for NodeMutRef<'_, T> {
    fn drop(&mut self) {
        trace!("drop MUT {}", self.node.key());
    }
}
