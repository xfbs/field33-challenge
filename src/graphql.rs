use async_graphql::*;

#[derive(SimpleObject)]
struct NodeRelationship {
    /// Internal Neo4j ID of the node
    _id: usize,
    /// Internal Neo4j ID of the start node
    _startId: usize,
    /// Internal Neo4j ID of the end node
    _endId: usize,
    relationshipType: String,
    startNode: GraphNode,
    endNode: GraphNode,
}

#[derive(SimpleObject)]
struct GraphNode {
    /// The internal Neo4j ID of the node
    _id: usize,
    /// Outward facing identifier of a node
    uri: usize,
    labels: Vec<String>,
    relationships: Vec<NodeRelationship>,
    numberOfNeighbors: usize,
}

struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn createGraphNode(&self, label: String) -> GraphNode {
        todo!()
    }

    async fn deleteGraphNode(&self, uri: usize) -> usize {
        todo!()
    }

    async fn createNodeRelationship(
        &self,
        startNodeUri: usize,
        endNodeUri: usize,
        relationshipType: String,
    ) -> NodeRelationship {
        todo!()
    }
}
