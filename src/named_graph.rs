use std::collections::HashMap;
use std::fmt;

use errors::GrammarError;
use parser_data::ElementIndex;

use crate::named_graph::GraphError::{IndexOutOfBounds, NodeAlreadyExistsError, NodeDoesNotExistsError};
use crate::simple_graph::{EdgeIndex, Graph, NodeData, NodeIndex, Successors};

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


pub struct GraphNamedNodes<Ta>{
    graph:Graph<Ta>,
    pub names: HashMap::<ElementIndex, NodeIndex>,
}

impl<Ta> GraphNamedNodes<Ta> {
    

    pub fn new()-> GraphNamedNodes<Ta>{
        GraphNamedNodes{graph:Graph::new(), names: HashMap::new()}
    }
    pub fn add_node(&mut self,node_name:ElementIndex ,data:Ta) -> Result<NodeIndex,GrammarError> {
        if self.names.contains_key(&node_name){
            return Err(GrammarError::GraphNodeAlreadyExistsError {node_name})
        }
        let index=self.graph.add_node(data);
        self.names.insert(node_name, index);
        Ok(index)
    }

    fn get_index_of_node(&self,node_name:ElementIndex)->Result<NodeIndex,GrammarError>{
        Ok(*self.names.get(&node_name).ok_or(GrammarError::GraphNodeDoesNotExistsError {node_name})? as NodeIndex)
    }

    pub fn get_node_by_index(&self, index:NodeIndex)->Result<&NodeData<Ta>,GrammarError>{
        if index>self.graph.nodes.len()-1{
            return Err(GrammarError::GraphIndexOutOfBounds {index})
        }
        Ok(&self.graph.nodes[index])
    }
    pub fn get_node_mut_by_index(&mut self, index:NodeIndex)->Result<&mut NodeData<Ta>,GrammarError>{
        if index>self.graph.nodes.len()-1{
            return Err(GrammarError::GraphIndexOutOfBounds {index})
        }
        Ok(&mut self.graph.nodes[index])
    }

    pub fn get_node(&self, node_name:ElementIndex)->Result<&NodeData<Ta>,GrammarError>{
        let index=self.get_index_of_node(node_name)?;
        Ok(&self.graph.nodes[index])
    }
    pub fn get_node_mut(&mut self, node_name:ElementIndex)->Result<&mut NodeData<Ta>,GrammarError>{
        let index=self.get_index_of_node(node_name)?;
        Ok(&mut self.graph.nodes[index])
    }
    pub fn add_edge(&mut self, source: ElementIndex, target: ElementIndex) -> Result<(),GrammarError>{
        let source_index=self.get_index_of_node(source)?;
        let target_index= self.get_index_of_node(target)?;
        self.graph.add_edge(source_index,target_index);
        Ok(())
    }

    pub fn successor_indices(&self, node_name: ElementIndex) -> Result<Successors<Ta>,GrammarError> {
        let index=self.get_index_of_node(node_name)?;
        Ok(self.graph.successors(index))
    }

    pub fn successors(&self, node_name: ElementIndex) -> Result<SuccessorsData<Ta>,GrammarError> {
        let index=self.get_index_of_node(node_name)?;
        let first_outgoing_edge = self.graph.nodes[index].first_outgoing_edge;
        Ok(SuccessorsData { graph: self, current_edge_index: first_outgoing_edge })
    }

}
pub struct SuccessorsData<'graph,Ta> {
    graph: &'graph GraphNamedNodes<Ta>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph,Ta> Iterator for SuccessorsData<'graph,Ta> {
    type Item = (NodeIndex,&'graph NodeData<Ta>);

    fn next(&mut self) -> Option<(NodeIndex,&'graph NodeData<Ta>)> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let ggraph: &Graph<Ta>= &self.graph.graph;
                let nodes: &Vec<NodeData<Ta>>=&(ggraph.nodes);
                let edge = &ggraph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some((edge.target,&nodes[edge.target]))
            }
        }
    }
}