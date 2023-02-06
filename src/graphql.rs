/// GraphQL types and implementations
use anyhow::anyhow;
use async_graphql::{types::ID, *};
use bolt_client::Metadata;
use bolt_proto::{
    message::Success,
    value::{Node, Relationship},
};
use deadpool_bolt::{Object, Pool};
use rand::{thread_rng, Rng};
use std::collections::{BTreeMap, BTreeSet};

/// Describes the relationship between two nodes (verices)
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
    #[graphql(name = "_endId")]
    pub end_id: i64,
    /// Type of relationship
    #[graphql(name = "relationshipType")]
    pub relationship_type: String,
}

/// Fetch a single GraphNode
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

/// Fetch all nodes
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

/// Describes a single node
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
    /// Create a new graph node with the given label
    ///
    /// Will allocate a new, random URI for the node
    #[graphql(name = "createGraphNode")]
    pub async fn create_graph_node(&self, ctx: &Context<'_>, label: String) -> Result<GraphNode> {
        let mut conn = ctx.data::<Pool>()?.get().await?;

        // generate random uri
        let uri = format!("http://example.com/entity/{}", thread_rng().gen::<u32>());

        // insert new node
        // cannot set label dynamically: https://github.com/neo4j/neo4j/issues/4334
        // this here is a potential injection vulnerability.
        let params = [("uri", uri)].into_iter().collect();
        let response = conn
            .run(
                &format!("CREATE (n:{label} {{ uri: $uri }}) RETURN ID(n)"),
                Some(params),
                None,
            )
            .await?;
        Success::try_from(response)?;
        let metadata = Metadata::from_iter(vec![("n", 1)]);
        let (records, response) = conn.pull(Some(metadata)).await?;
        Success::try_from(response)?;
        let id: i64 = records
            .get(0)
            .ok_or(anyhow!("Missing record for node insertion"))?
            .fields()
            .get(0)
            .ok_or(anyhow!("Missing field for node insertion record"))?
            .clone()
            .try_into()?;

        // fetch node
        node_get(&mut conn, id).await
    }

    /// Delete the graph node with the given URI
    #[graphql(name = "deleteGraphNode")]
    pub async fn delete_graph_node(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "nodeUri")] uri: String,
    ) -> Result<usize> {
        let mut conn = ctx.data::<Pool>()?.get().await?;

        let params = [("uri", uri)].into_iter().collect();
        let response = conn
            .run("MATCH (n{uri: $uri}) DETACH DELETE n", Some(params), None)
            .await?;
        Success::try_from(response)?;

        // not sure what this should return?
        Ok(0)
    }

    /// Create a relationship between two nodes by their URI
    #[graphql(name = "createNodeRelationship")]
    pub async fn create_node_relationship(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "startNodeUri")] start_node_uri: String,
        #[graphql(name = "endNodeUri")] end_node_uri: String,
        #[graphql(name = "relationshipType")] relationship_type: String,
    ) -> Result<NodeRelationship> {
        let mut conn = ctx.data::<Pool>()?.get().await?;

        let params = [("start", start_node_uri), ("end", end_node_uri)]
            .into_iter()
            .collect();
        let response = conn
            .run(
                &format!("MATCH (start{{uri:$start}}), (end{{uri:$end}}) CREATE (start)-[r:{relationship_type}]->(end) RETURN r"),
                Some(params),
                None,
            )
            .await?;
        Success::try_from(response)?;
        let metadata = Metadata::from_iter(vec![("n", 1)]);
        let (records, response) = conn.pull(Some(metadata)).await?;
        Success::try_from(response)?;
        let relation: Relationship = records
            .get(0)
            .ok_or(anyhow!("No records returned from insertion"))?
            .fields()
            .get(0)
            .ok_or(anyhow!("No fields returned in record"))?
            .clone()
            .try_into()?;
        let node_relationship = NodeRelationship {
            id: relation.rel_identity(),
            start_id: relation.start_node_identity(),
            end_id: relation.end_node_identity(),
            relationship_type: relation.rel_type().to_string(),
        };

        Ok(node_relationship)
    }
}
