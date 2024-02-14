use std::collections::HashMap;
use crate::simple_graph::{EdgeIndex, Graph, NodeData, NodeIndex, Successors};

use std::fmt;
use crate::named_graph::GraphError::{NodeAlreadyExistsError, NodeDoesNotExistsError,IndexOutOfBounds};

#[derive(Debug)]
pub enum GraphError {
    NodeAlreadyExistsError { node_name:String},
    NodeDoesNotExistsError { node_name:String},
    IndexOutOfBounds {index:NodeIndex}
}

impl std::error::Error for GraphError {}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeAlreadyExistsError { node_name } => write!(f, "Node with the name \"{}\" already exists", node_name),
            NodeDoesNotExistsError{node_name} => write!(f, "Node \"{}\" does not exist", node_name),
            IndexOutOfBounds {index}=>write!(f,"Index \"{}\" out of bounds", index)
        }
    }
}


pub struct GraphNamedNodes<T>{
    graph:Graph<T>,
    pub names: HashMap::<String, NodeIndex>,
}

impl<T> GraphNamedNodes<T>{

    pub fn nodes(&self)->&Vec<NodeData<T>>{
        &self.graph.nodes
    }

    pub fn new()-> GraphNamedNodes<T>{
        GraphNamedNodes{graph:Graph::new(), names: HashMap::new()}
    }
    pub fn add_node(&mut self,node_name: String,data:T) -> Result<NodeIndex,GraphError> {
        if self.names.contains_key(&node_name){
            return Err(NodeAlreadyExistsError {node_name})
        }
        let index=self.graph.add_node(data);
        self.names.insert(node_name, index);
        Ok(index)
    }

    fn get_index_of_node(&self,node_name:&str)->Result<NodeIndex,GraphError>{
        Ok(*self.names.get(node_name).ok_or(NodeDoesNotExistsError {node_name:String::from(node_name)})? as NodeIndex)
    }

    pub fn get_node_by_index(&self, index:NodeIndex)->Result<&NodeData<T>,GraphError>{
        if index>self.graph.nodes.len()-1{
            return Err(IndexOutOfBounds {index})
        }
        Ok(&self.graph.nodes[index])
    }
    pub fn get_node_mut_by_index(&mut self, index:NodeIndex)->Result<&mut NodeData<T>,GraphError>{
        if index>self.graph.nodes.len()-1{
            return Err(IndexOutOfBounds {index})
        }
        Ok(&mut self.graph.nodes[index])
    }

    pub fn get_node(&self, node_name:&str)->Result<&NodeData<T>,GraphError>{
        let index=self.get_index_of_node(node_name)?;
        Ok(&self.graph.nodes[index])
    }
    pub fn get_node_mut(&mut self, node_name:&str)->Result<&mut NodeData<T>,GraphError>{
        let index=self.get_index_of_node(node_name)?;
        Ok(&mut self.graph.nodes[index])
    }
    pub fn add_edge(&mut self, source: &str, target: &str) -> Result<(),GraphError>{
        let source_index=self.get_index_of_node(source)?;
        let target_index= self.get_index_of_node(target)?;
        self.graph.add_edge(source_index,target_index);
        Ok(())
    }

    pub fn successor_indices(&self, node_name: &str) -> Result<Successors<T>,GraphError> {
        let index=self.get_index_of_node(node_name)?;
        Ok(self.graph.successors(index))
    }

    pub fn successors(&self, node_name: &str) -> Result<SuccessorsData<T>,GraphError> {
        let index=self.get_index_of_node(node_name)?;
        let first_outgoing_edge = self.graph.nodes[index].first_outgoing_edge;
        Ok(SuccessorsData { graph: self, current_edge_index: first_outgoing_edge })
    }

}
pub struct SuccessorsData<'graph,T> {
    graph: &'graph GraphNamedNodes<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph,T> Iterator for SuccessorsData<'graph,T> {
    type Item = (NodeIndex,&'graph NodeData<T>);

    fn next(&mut self) -> Option<(NodeIndex,&'graph NodeData<T>)> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let ggraph: &Graph<T>= &self.graph.graph;
                let nodes: &Vec<NodeData<T>>=&(ggraph.nodes);
                let edge = &ggraph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some((edge.target,&nodes[edge.target]))
            }
        }
    }
}