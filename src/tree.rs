use simple_graph::NodeIndex;
use std::fmt;
use tree::TreeError::{ChildDoesNotExists, NodeDoesNotExists, NodeWasRemoved};
use ::tree;

pub type NodeId = usize;

#[derive(Debug)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
    free_node_indexes: Vec<NodeIndex>,

}

#[derive(Debug)]
pub struct Node<T> {
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    pub data: T,
}


impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree { nodes: vec![], free_node_indexes: vec![] }
    }
    pub fn add_node(&mut self, data: T, parent: Option<NodeId>) -> Result<NodeId, TreeError> {
        // Get the next free index
        let mut reused_index = false;
        let next_index = match self.free_node_indexes.pop() {
            None => { self.nodes.len() }
            Some(i) => {
                reused_index = true;
                i
            }
        };
        match parent {
            None => {
                let new_node = Node {
                    parent: None,
                    first_child: None,
                    last_child: None,
                    previous_sibling: None,
                    next_sibling: None,
                    data,
                };
                if reused_index {
                    self.nodes[next_index] = new_node;
                } else {
                    self.nodes.push(new_node);
                }
            }
            Some(parent_id) => {
                let parent_node = self.get_node_mut(parent_id)?;

                let new_node = Node {
                    parent: Some(parent_id),
                    first_child: None,
                    last_child: None,
                    previous_sibling: parent_node.last_child,
                    next_sibling: None,
                    data,
                };

                let last_child_index = parent_node.last_child;
                if let None = parent_node.first_child {
                    parent_node.first_child = Some(next_index);
                }
                parent_node.last_child = Some(next_index);
                if let Some(lci) = last_child_index {
                    self.nodes[lci].next_sibling = Some(next_index);
                }

                Some(next_index);
                if reused_index {
                    self.nodes[next_index] = new_node;
                } else {
                    self.nodes.push(new_node);
                }
            }
        }

        Ok(next_index)
    }
    



    pub fn get_descendants(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut descendants = vec![];
        let children = self.get_children(node_id);
        descendants.extend(&children);
        for child in children {
            let child_descendants = self.get_descendants(child);
            descendants.extend(child_descendants);
        };
        descendants
    }

    pub fn get_children(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut children = vec![];
        let node = &self.nodes[node_id];
        let mut cur_children_ix = node.first_child;
        while let Some(child_index) = cur_children_ix {
            let cur_children = &self.nodes[child_index];
            children.push(child_index);
            cur_children_ix = cur_children.next_sibling;
        }

        children
    }
    
    pub fn get_nth_child(&self, node: &Node<T>, nth:usize ) -> Result<&Node<T>,TreeError> {
        let children=Children{tree:self,current_child:node.first_child};
        for (i,child) in children.enumerate() {
            if i==nth{
                return Ok(child)
            }
        }
        Err(ChildDoesNotExists {child_nth:nth})
    }pub fn get_nth_child_or_none(&self, node: &Node<T>, nth:usize ) -> Option<&Node<T>> {
        let children=Children{tree:self,current_child:node.first_child};
        for (i,child) in children.enumerate() {
            if i==nth{
                return Some(child)
            }
        }
        None
    }
    pub fn get_by_path_or_none(&self, source: NodeId, ids: impl Iterator<Item = usize>) ->Result<Option<&Node<T>>,TreeError>{
        let mut node=self.get_node(source)?;
        
        for id in ids {
            if let Some(nodee)=self.get_nth_child_or_none(node,id){
                node=nodee;
            }else{
                return Ok(None);
            }
        }
        
        Ok(Some(node))
    }

    pub fn get_node(&self, node_id: NodeId) -> Result<&Node<T>, TreeError> {
        if node_id >= self.nodes.len() {
            return Err(NodeDoesNotExists { node_id });
        }
        if self.free_node_indexes.contains(&node_id) {
            return Err(NodeWasRemoved { node_id });
        }
        Ok(&self.nodes[node_id])
    }
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut Node<T>, TreeError> {
        if node_id >= self.nodes.len() {
            return Err(NodeDoesNotExists { node_id });
        }
        if self.free_node_indexes.contains(&node_id) {
            return Err(NodeWasRemoved { node_id });
        }
        Ok(&mut self.nodes[node_id])
    }

    pub fn remove_branch(&mut self, node_id: NodeId) {
        let node = &self.nodes[node_id];
        let parent_id_maybe = node.parent;
        let mut nodes_to_free = self.get_descendants(node_id);
        nodes_to_free.push(node_id);

        let next_sibling_id = node.next_sibling;
        let previous_sibling_id = node.previous_sibling;


        if let Some(parent_id) = parent_id_maybe {
            let parent = &mut self.nodes[parent_id];
            if parent.first_child == Some(node_id) {
                parent.first_child = next_sibling_id;
            }
            if parent.last_child == Some(node_id) {
                parent.last_child = previous_sibling_id
            }
        }
        if let Some(pre_sibling) = previous_sibling_id {
            let pre = &mut self.nodes[pre_sibling];
            pre.next_sibling = next_sibling_id;
        }
        if let Some(next_sibling) = next_sibling_id {
            let next = &mut self.nodes[next_sibling];
            next.previous_sibling = previous_sibling_id;
        }
        self.free_node_indexes.extend(nodes_to_free);
    }
}


pub struct Children<'tree, T> {
    tree: &'tree Tree<T>,
    current_child: Option<NodeId>,
}

impl<'tree, T> Iterator for Children<'tree, T> {
    type Item = &'tree Node<T>;

    fn next(&mut self) ->Option<&'tree Node<T>> {
        match self.current_child {
            None => None,
            Some(node_id) => {
                let child= &self.tree.nodes[node_id];
                let next_id = child.next_sibling;
                self.current_child = next_id;
                Some(child)
            }
        }
    }
}

#[derive(Debug)]
pub enum TreeError {
    NodeDoesNotExists { node_id: NodeId },
    NodeWasRemoved { node_id: NodeId },
    ChildDoesNotExists { child_nth: usize },
}

impl std::error::Error for TreeError {}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeDoesNotExists { node_id } => write!(f, "{} does not exists", node_id),
            NodeWasRemoved { node_id } => write!(f, "{} was removed", node_id),
            ChildDoesNotExists {child_nth}=> write!(f, "{} does not exist", child_nth),
        }
    }
}


#[cfg(test)]
mod tests {
    use tree::Tree;

    #[test]
    fn test_tree_add_node() {
        let mut tree= Tree::new();
        let node=tree.add_node("test",None).unwrap();
        let res = tree.get_node(node).unwrap().data;
        assert_eq!("test", res);
        
    }
    #[test]
    fn test_tree_add_delete_node() {
        let mut tree= Tree::new();
        let node=tree.add_node("test",None).unwrap();
        tree.remove_branch(node);
        
        let res = tree.get_node(node);
        assert!( res.is_err());

    }
    #[test]
    fn test_tree_add_delete_add_node() {
        let mut tree= Tree::new();
        let node=tree.add_node("remove",None).unwrap();
        tree.remove_branch(node);
        let node2=tree.add_node("test",None).unwrap();

        let res = tree.get_node(node2).unwrap().data;
        assert_eq!("test", res);
    }
    #[test]
    fn test_tree_add_delete_add_node2() {
        //TODO Missing feature
        let mut tree= Tree::new();
        let node_deleted=tree.add_node("remove",None).unwrap();
        tree.remove_branch(node_deleted);
        let node =tree.add_node("test", None).unwrap();

        let res = tree.get_node(node_deleted);
        assert!( res.is_err());
    }

    #[test]
    fn test_tree_add_node_child() {
        let mut tree= Tree::new();
        let node=tree.add_node("test",None).unwrap();
        let _= tree.add_node("child",Some(node)).unwrap();
        let childs=tree.get_children(node);
        let child_data= tree.get_node(childs[0]).unwrap().data;
        assert_eq!("child", child_data);

    }

    #[test]
    fn test_tree_get_by_path() {
        let mut tree= Tree::new();
        let node=tree.add_node("test",None).unwrap();
        let _= tree.add_node("child",Some(node)).unwrap();
        let child= tree.get_by_path_or_none(node,vec![0].into_iter()).unwrap().unwrap();

        let child_data= child.data;
        assert_eq!("child", child_data);

    }
}
