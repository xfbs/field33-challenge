use crate::graphql::{Mutation, Query};
use crate::options::Options;
use anyhow::{anyhow, Context, Result};
use async_graphql::{extensions::Tracing, http::GraphiQLSource, EmptySubscription};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    body::Body,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router, Server,
};
use bolt_client::*;
use bolt_proto::{message::*, value::*, version::*, Message, Value};
use deadpool_bolt::{bolt_client::Metadata, Manager, Pool};
use tokio::io::BufStream;
use tokio_util::compat::*;

type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

async fn graphql(Extension(schema): Extension<Schema>, request: GraphQLRequest) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

impl Options {
    pub async fn database(&self) -> Result<Pool> {
        let domain_name = match self.database.scheme() {
            "bolt" => None,
            "bolts" => Some(
                self.database
                    .domain()
                    .ok_or(anyhow!("Missing domain name for database URL"))?
                    .to_string(),
            ),
            scheme => return Err(anyhow!("Unrecognized protocol: {scheme}")),
        };
        let socket_addrs = self.database.socket_addrs(|| Some(7687))?;

        let manager = Manager::new(
            &socket_addrs[..],
            domain_name,
            [V4_2, V4_1, 0, 0],
            Metadata::from_iter(vec![
                (
                    "user_agent",
                    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
                ),
                ("scheme", "basic"),
                ("principal", &self.username),
                ("credentials", &self.password),
            ]),
        )
        .await?;

        // start connection pool
        let pool = Pool::builder(manager).build()?;

        // make sure we are connected
        let mut conn = pool.get().await?;
        drop(conn);

        Ok(pool)
    }

    pub async fn service(&self, database: Pool) -> Result<Router<(), Body>> {
        let query = Query {
            database: database.clone(),
        };
        let mutation = Mutation {
            database: database.clone(),
        };
        let schema = Schema::build(query, mutation, EmptySubscription)
            .extension(Tracing)
            .finish();
        let router = Router::new()
            .route("/graphql", post(graphql))
            .route("/", get(graphiql))
            .layer(Extension(schema));
        Ok(router)
    }

    pub async fn run(self) -> Result<()> {
        let database = self.database().await?;
        let router = self.service(database.clone()).await?;

        Server::bind(&self.listen)
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }
}
