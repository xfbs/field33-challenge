use async_graphql::{types::ID, *};
use bolt_client::Metadata;
use bolt_proto::{
    message::Success,
    value::{Node, Relationship},
};
use deadpool_bolt::Pool;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(SimpleObject, PartialOrd, Ord, Clone, Debug, PartialEq, Eq)]
pub struct NodeRelationship {
    /// Internal Neo4j ID of the node
    #[graphql(name = "_id")]
    pub id: i64,
    /// Internal Neo4j ID of the start node
    #[graphql(name = "_startId")]
    pub start_id: i64,
    /// Internal Neo4j ID of the end node
    #[graphql(name = "_endID")]
    pub end_id: i64,
    /// Type of relationship
    #[graphql(name = "relationshipType")]
    pub relationship_type: String,
}

#[derive(SimpleObject)]
pub struct GraphNode {
    /// Internal Neo4j ID of the node
    #[graphql(name = "_id")]
    pub id: ID,
    /// Outward facing identifier of a node
    pub uri: ID,
    /// Labels of this node
    pub labels: Vec<String>,
    /// Relationships with other nodes
    pub relationships: BTreeSet<NodeRelationship>,
    /// Number of neighbors
    #[graphql(name = "numberOfNeighbors")]
    pub number_of_neighbors: usize,
}

pub struct Query;

#[Object]
impl Query {
    pub async fn graphNodes(&self, ctx: &Context<'_>) -> Result<Vec<GraphNode>> {
        let mut conn = ctx.data::<Pool>()?.get().await?;

        // get all nodes
        let response = conn.run("MATCH (n) RETURN n", None, None).await?;
        Success::try_from(response)?;
        let metadata = Metadata::from_iter(vec![("n", -1)]);
        let (records, response) = conn.pull(Some(metadata)).await?;
        Success::try_from(response)?;
        let mut nodes = BTreeMap::new();
        for record in &records {
            let field = record.fields().get(0).unwrap();
            let node: Node = field.clone().try_into()?;
            let uri: String = node
                .properties()
                .get("uri")
                .unwrap()
                .clone()
                .try_into()
                .unwrap();
            nodes.insert(
                node.node_identity(),
                GraphNode {
                    id: ID(node.node_identity().to_string()),
                    uri: ID(uri),
                    labels: node.labels().to_vec(),
                    relationships: Default::default(),
                    number_of_neighbors: 0,
                },
            );
        }

        let response = conn.run("MATCH (n)-[r]->(d) RETURN r", None, None).await?;
        Success::try_from(response)?;
        let metadata = Metadata::from_iter(vec![("n", -1)]);
        let (records, response) = conn.pull(Some(metadata)).await?;
        Success::try_from(response)?;
        for record in &records {
            let relation: Relationship = record.fields().get(0).unwrap().clone().try_into()?;
            let node_relationship = NodeRelationship {
                id: relation.rel_identity(),
                start_id: relation.start_node_identity(),
                end_id: relation.end_node_identity(),
                relationship_type: relation.rel_type().to_string(),
            };
            let mut start = nodes.get_mut(&relation.start_node_identity()).unwrap();
            start.relationships.insert(node_relationship.clone());
            drop(start);
            let mut end = nodes.get_mut(&relation.end_node_identity()).unwrap();
            end.relationships.insert(node_relationship.clone());
            drop(end);
        }

        Ok(nodes.into_values().collect())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn create_graph_node(&self, label: String) -> GraphNode {
        todo!()
    }

    pub async fn delete_graph_node(&self, uri: usize) -> usize {
        todo!()
    }

    pub async fn create_node_relationship(
        &self,
        start_node_uri: usize,
        end_node_uri: usize,
        relationship_type: String,
    ) -> NodeRelationship {
        todo!()
    }
}
