use anyhow::anyhow;
use async_graphql::{types::ID, *};
use bolt_client::Metadata;
use bolt_proto::{
    message::Success,
    value::{Node, Relationship},
};
use deadpool_bolt::{Object, Pool};
use std::collections::{BTreeMap, BTreeSet};

#[derive(SimpleObject, PartialOrd, Ord, Clone, Debug, PartialEq, Eq)]
#[graphql(complex)]
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

async fn node_get(conn: &mut Object, id: i64) -> Result<GraphNode> {
    let params = [("id", id)].into_iter().collect();
    let response = conn
        .run("MATCH (n) WHERE ID(n) = $id RETURN n", Some(params), None)
        .await?;
    Success::try_from(response)?;
    let metadata = Metadata::from_iter(vec![("n", 1)]);
    let (records, response) = conn.pull(Some(metadata)).await?;
    Success::try_from(response)?;
    let record = records.first().unwrap();
    let node: Node = record.fields().get(0).unwrap().clone().try_into()?;
    let uri: String = node
        .properties()
        .get("uri")
        .ok_or(anyhow!(
            "Missing URI property on node {}",
            node.node_identity()
        ))?
        .clone()
        .try_into()?;
    let mut node = GraphNode {
        id: ID(node.node_identity().to_string()),
        uri: ID(uri),
        labels: node.labels().to_vec(),
        relationships: Default::default(),
        number_of_neighbors: 0,
    };

    let params = [("id", id)].into_iter().collect();
    let response = conn
        .run(
            "MATCH (n)-[r]->(d) WHERE ID(n) = $id OR ID(d) = $id RETURN r",
            Some(params),
            None,
        )
        .await?;
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
        node.relationships.insert(node_relationship.clone());
        node.number_of_neighbors = node.relationships.len();
    }

    Ok(node)
}

async fn node_all(conn: &mut Object) -> Result<Vec<GraphNode>> {
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
            .ok_or(anyhow!(
                "Missing URI property on node {}",
                node.node_identity()
            ))?
            .clone()
            .try_into()?;
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
        start.number_of_neighbors = start.relationships.len();
        drop(start);
        let mut end = nodes.get_mut(&relation.end_node_identity()).unwrap();
        end.relationships.insert(node_relationship.clone());
        end.number_of_neighbors = end.relationships.len();
        drop(end);
    }

    Ok(nodes.into_values().collect())
}

#[ComplexObject]
impl NodeRelationship {
    #[graphql(name = "startNode")]
    pub async fn start_node(&self, ctx: &Context<'_>) -> Result<GraphNode> {
        let mut conn = ctx.data::<Pool>()?.get().await?;
        node_get(&mut conn, self.start_id).await
    }

    #[graphql(name = "endNode")]
    pub async fn end_node(&self, ctx: &Context<'_>) -> Result<GraphNode> {
        let mut conn = ctx.data::<Pool>()?.get().await?;
        node_get(&mut conn, self.end_id).await
    }
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
    #[graphql(name = "graphNodes")]
    pub async fn graph_nodes(&self, ctx: &Context<'_>) -> Result<Vec<GraphNode>> {
        let mut conn = ctx.data::<Pool>()?.get().await?;
        node_all(&mut conn).await
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
