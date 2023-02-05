use crate::graphql::{Mutation, Query};
use crate::options::Options;
use anyhow::{anyhow, Context, Result};
use async_graphql::{http::GraphiQLSource, EmptySubscription};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    body::Body,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router, Server,
};
use bolt_client::*;
use bolt_proto::{message::*, value::*, version::*, Message, Value};
use tokio::io::BufStream;
use tokio_util::compat::*;

type BoltClient = Client<Compat<BufStream<Stream>>>;
type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

async fn graphql(Extension(schema): Extension<Schema>, request: GraphQLRequest) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

impl Options {
    pub async fn connect(&self) -> Result<BoltClient> {
        match self.database.scheme() {
            "bolt" => {
                // connect to database server
                let socket_addrs = self.database.socket_addrs(|| Some(7687))?;
                let stream = Stream::connect(&socket_addrs[..], None as Option<String>).await?;

                // create client
                let stream = BufStream::new(stream).compat();
                let mut client = Client::new(stream, &[V4_3, V4_2, 0, 0]).await?;

                // authenticate
                let response: Message = client
                    .hello(Metadata::from_iter(vec![
                        (
                            "user_agent",
                            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
                        ),
                        ("scheme", "basic"),
                        ("principal", &self.username),
                        ("credentials", &self.password),
                    ]))
                    .await?;
                Success::try_from(response)?;

                Ok(client)
            }
            scheme => Err(anyhow!("Invalid scheme: {scheme}")),
        }
    }

    pub async fn service(&self) -> Result<Router<(), Body>> {
        let schema = Schema::build(Query, Mutation, EmptySubscription).finish();
        let router = Router::new()
            .route("/graphql", post(graphql))
            .route("/", get(graphiql))
            .layer(Extension(schema));
        Ok(router)
    }

    pub async fn run(self) -> Result<()> {
        let client = self.connect().await?;
        let router = self.service().await?;

        Server::bind(&self.listen)
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }
}
