//initial idea stolen from here:
//http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
enum ThingWithNextEdge {
    Node(NodeIndex),
    Edge(EdgeIndex),
}

pub struct Graph<T> {
    pub nodes: Vec<NodeData<T>>,
    pub edges: Vec<EdgeData>,
}

pub type NodeIndex = usize;

pub struct NodeData<T> {
    pub first_outgoing_edge: Option<EdgeIndex>,
    pub data: T,
}


pub type EdgeIndex = usize;
pub type EdgeId = usize;

pub struct EdgeData {
    pub target: NodeIndex,
    pub id: EdgeId,
    pub next_outgoing_edge: Option<EdgeIndex>,
}

impl<T> Graph<T> {
    pub fn new() -> Graph<T> {
        Graph { nodes: vec![], edges: vec![] }
    }

    pub fn add_node(&mut self, data: T) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(NodeData { first_outgoing_edge: None, data });
        index
    }


    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) {
        let edge_index: EdgeIndex = self.edges.len();
        let node_data = &mut self.nodes[source];
        let last_edge_index = node_data.first_outgoing_edge;
        let new_edge_id = match last_edge_index {
            None => { 0 }
            Some(index) => { self.edges.get(index).unwrap().id + 1 }
        };
        self.edges.push(EdgeData {
            target,
            next_outgoing_edge: node_data.first_outgoing_edge,
            id: new_edge_id,
        });
        node_data.first_outgoing_edge = Some(edge_index);
    }

    pub fn try_add_edge_with_id(&mut self, source: NodeIndex, target: NodeIndex, id: usize) -> bool {
        
        let connected_edges = self.connected_edges(source);
        let mut previous: ThingWithNextEdge = ThingWithNextEdge::Node(source);
        let mut next_edge_index = self.nodes[source].first_outgoing_edge;
        for connected_edge_index in connected_edges {
            let connected_edge = &self.edges[connected_edge_index];
            if connected_edge.id == id {
                return false;
            } else if connected_edge.id > id {
                previous = ThingWithNextEdge::Edge(connected_edge_index);
                next_edge_index = connected_edge.next_outgoing_edge;
                continue;
            }
            break;
        }
        let edge_index: EdgeIndex = self.edges.len();
        match previous {
            ThingWithNextEdge::Node(node) => { self.nodes[node].first_outgoing_edge = Some(edge_index) }
            ThingWithNextEdge::Edge(edge) => { self.edges[edge].next_outgoing_edge = Some(edge_index) }
        }

        self.edges.push(EdgeData {
            target,
            next_outgoing_edge: next_edge_index,
            id,
        });

        true
    }
    pub fn find_node_index(&self, source: NodeIndex, id: EdgeId) -> Option<NodeIndex> {
        let edges = self.connected_edges(source);
        for edge_index in edges {
            let edge = &self.edges[edge_index];
            if edge.id == id {
                return Some(edge.target);
            } else if edge.id < id {
                return None;
            }
        };
        None
    }
    pub fn find_node_index_by_path(&self, source: NodeIndex, ids: impl Iterator<Item = EdgeIndex>) -> Option<NodeIndex> {
        let mut cur_node_index= Some(source);
        for id in ids {
            match cur_node_index {
                None => {return None}
                Some(cni) => {cur_node_index=self.find_node_index(cni,id);}
            }
        }
        cur_node_index
    }

    pub fn find_node(&self, source: NodeIndex, id: EdgeId) -> Option<&NodeData<T>> {
        self.find_node_index(source, id).map(|index| &self.nodes[index])
    }
    
    pub fn find_node_by_path(&self, source: NodeIndex, ids: impl Iterator<Item = EdgeIndex>) -> Option<&NodeData<T>> {
        self.find_node_index_by_path(source, ids).map(|index| &self.nodes[index])
    }


    pub fn successors(&self, source: NodeIndex) -> Successors<T> {
        let first_outgoing_edge = self.nodes[source].first_outgoing_edge;
        Successors { graph: self, current_edge_index: first_outgoing_edge }
    }

    pub fn connected_edges(&self, source: NodeIndex) -> ConnectedEdges<T> {
        ConnectedEdges { graph: self, current_edge_index: self.nodes[source].first_outgoing_edge }
    }


    pub fn add_graph_at_node(&mut self, graph: Graph<T>, this: NodeIndex, other: NodeIndex) {
        let node_offset = self.nodes.len();
        let edge_offset = self.edges.len();
        for node in graph.nodes.into_iter() {
            let first_outgoing_edge: Option<EdgeIndex> = node.first_outgoing_edge.map(|ei| edge_offset + ei);
            self.nodes.push(NodeData { data: node.data, first_outgoing_edge });
        }

        for edge in graph.edges.into_iter() {
            let next_outgoing_edge: Option<EdgeIndex> = edge.next_outgoing_edge.map(|ei| edge_offset + ei);
            self.edges.push(EdgeData { target: node_offset + edge.target, next_outgoing_edge, id: edge.id })
        }
        self.add_edge(this, other + node_offset)
    }
}

pub struct Successors<'graph, T> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, T> Iterator for Successors<'graph, T> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge.target)
            }
        }
    }
}

pub struct ConnectedEdges<'graph, T> {
    graph: &'graph Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, T> Iterator for ConnectedEdges<'graph, T> {
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<EdgeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge_num)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::simple_graph::Graph;

    #[test]
    fn edge_id_add() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        graph.add_node("Mr.Mouse".to_string());
        graph.add_edge(0, 1);
        let res = graph.try_add_edge_with_id(0, 2, 0);
        assert!(!res);
    }

    #[test]
    fn edge_id_add2() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        graph.add_node("Mr.Mouse".to_string());
        graph.add_edge(0, 1);
        let res = graph.try_add_edge_with_id(0, 2, 1);
        assert!(res);
    }

    #[test]
    fn edge_id_add3() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        graph.add_node("Mr.Mouse".to_string());
        graph.add_node("Mrs.Mouse".to_string());
        graph.add_edge(0, 1);
        assert!(graph.try_add_edge_with_id(0, 3, 2));
        assert!(graph.try_add_edge_with_id(0, 2, 1))
    }

    #[test]
    fn edge_id_add4() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        assert!(graph.try_add_edge_with_id(0, 1, 0));
    }

    #[test]
    fn edge_id_add5() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        assert!(graph.try_add_edge_with_id(0, 1, 0));
        assert!(graph.try_add_edge_with_id(1, 0, 0));
    }

    #[test]
    fn edge_id_find() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        assert!(graph.try_add_edge_with_id(0, 1, 0));
        let res = &graph.find_node(0, 0).unwrap().data;
        assert_eq!("World",res);
    }
    #[test]
    fn edge_id_find2() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        assert!(graph.try_add_edge_with_id(0, 1, 0));
        let res = graph.find_node(0, 1);
        assert!(res.is_none());
    }
    #[test]
    fn edge_id_find3() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        let res = graph.find_node(0, 1);
        assert!(res.is_none());
    }
    #[test]
    fn edge_id_find4() {
        let mut graph = Graph::<String>::new();
        graph.add_node("Hello, ".to_string());
        graph.add_node("World".to_string());
        graph.add_node("Mr. Mouse".to_string());
        graph.add_node("Mrs. Mouse".to_string());
        assert!(graph.try_add_edge_with_id(0, 1, 0));
        assert!(graph.try_add_edge_with_id(0, 2, 10));
        assert_eq!("World",graph.find_node(0, 0).unwrap().data);
        assert!(graph.find_node(0, 1).is_none());
        assert_eq!("Mr. Mouse",graph.find_node(0, 10).unwrap().data);
        assert!(graph.try_add_edge_with_id(0, 3, 1));
        assert_eq!("Mrs. Mouse",graph.find_node(0, 1).unwrap().data);
    }
}





