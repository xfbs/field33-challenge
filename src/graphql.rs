use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
      "partner"
    }
}
