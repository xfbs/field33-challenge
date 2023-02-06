use async_graphql::*;
use bolt_client::Metadata;
use bolt_proto::{message::Success, value::Node};
use deadpool_bolt::Pool;

#[derive(SimpleObject)]
pub struct NodeRelationship {
    /// Internal Neo4j ID of the node
    pub _id: i64,
    /// Internal Neo4j ID of the start node
    #[graphql(name = "startId")]
    pub start_id: usize,
    /// Internal Neo4j ID of the end node
    #[graphql(name = "endId")]
    pub end_id: usize,
    #[graphql(name = "relationshipType")]
    pub relationship_type: String,
    #[graphql(name = "startNode")]
    pub start_node: GraphNode,
    #[graphql(name = "endNode")]
    pub end_node: GraphNode,
}

#[derive(SimpleObject)]
pub struct GraphNode {
    /// The internal Neo4j ID of the node
    pub _id: i64,
    /// Outward facing identifier of a node
    pub uri: usize,
    pub labels: Vec<String>,
    pub relationships: Vec<NodeRelationship>,
    #[graphql(name = "numberOfNeighbors")]
    pub number_of_neighbors: usize,
}

pub struct Query {
    pub database: Pool,
}

#[Object]
impl Query {
    pub async fn graphNodes(&self) -> Result<Vec<GraphNode>> {
        let mut conn = self.database.get().await?;
        let response = conn.run("MATCH (n) RETURN n", None, None).await?;
        Success::try_from(response)?;
        let metadata = Metadata::from_iter(vec![("n", -1)]);
        let (records, response) = conn.pull(Some(metadata)).await?;
        Success::try_from(response)?;
        let mut nodes = vec![];
        for record in &records {
            for field in record.fields() {
                let node: Node = field.clone().try_into()?;
                nodes.push(GraphNode {
                    _id: node.node_identity(),
                    uri: 0,
                    labels: node.labels().to_vec(),
                    relationships: vec![],
                    number_of_neighbors: 0,
                });
            }
        }

        Ok(nodes)
    }
}

pub struct Mutation {
    pub database: Pool,
}

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
