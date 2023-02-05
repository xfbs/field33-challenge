use async_graphql::*;

#[derive(SimpleObject)]
pub struct NodeRelationship {
    /// Internal Neo4j ID of the node
    pub _id: usize,
    /// Internal Neo4j ID of the start node
    pub _startId: usize,
    /// Internal Neo4j ID of the end node
    pub _endId: usize,
    pub relationshipType: String,
    pub startNode: GraphNode,
    pub endNode: GraphNode,
}

#[derive(SimpleObject)]
pub struct GraphNode {
    /// The internal Neo4j ID of the node
    pub _id: usize,
    /// Outward facing identifier of a node
    pub uri: usize,
    pub labels: Vec<String>,
    pub relationships: Vec<NodeRelationship>,
    pub numberOfNeighbors: usize,
}

pub struct Query;

#[Object]
impl Query {
    pub async fn graphNodes(&self) -> Vec<GraphNode> {
        vec![]
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn createGraphNode(&self, label: String) -> GraphNode {
        todo!()
    }

    pub async fn deleteGraphNode(&self, uri: usize) -> usize {
        todo!()
    }

    pub async fn createNodeRelationship(
        &self,
        startNodeUri: usize,
        endNodeUri: usize,
        relationshipType: String,
    ) -> NodeRelationship {
        todo!()
    }
}
