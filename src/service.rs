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
use bolt_proto::version::*;
use deadpool_bolt::{bolt_client::Metadata, Manager, Pool};

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
        let socket_addrs = self
            .database
            .socket_addrs(|| Some(7687))
            .context("Looking up socket addresses for database host")?;

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
        .await
        .context("Launching database connection pool manager")?;

        // start connection pool
        let pool = Pool::builder(manager)
            .build()
            .context("Building database connection pool manager")?;

        // make sure we are connected
        let conn = pool
            .get()
            .await
            .context("Fetching connection from connection pool manager")?;
        drop(conn);

        Ok(pool)
    }

    pub async fn service(&self, database: Pool) -> Result<Router<(), Body>> {
        let schema = Schema::build(Query, Mutation, EmptySubscription)
            .extension(Tracing)
            .data(database.clone())
            .finish();
        let router = Router::new()
            .route("/graphql", post(graphql))
            .route("/", get(graphiql))
            .layer(Extension(schema));
        Ok(router)
    }

    pub async fn run(self) -> Result<()> {
        let database = self.database().await.context("Connecting to database")?;
        let router = self
            .service(database.clone())
            .await
            .context("Launching service")?;

        Server::bind(&self.listen)
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }
}
