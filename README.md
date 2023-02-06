# Field33 Challenge

GraphQL backend for abstract graph editor.

## Statistics

Execution time: ~8 hours.

## Running

Dependencies:

- docker
- make
- cargo

Launch it:

```
# launch database using docker
$ make database
# launch service, listening on localhost:8000 by default
$ cargo run
```

Connect to GraphiQL at [http://127.0.0.1:8000]().

## Queries

Get all graph nodes:

```
query GetGraphNodes {
    graphNodes {
        _id
        uri
        labels
        relationships {
            _startId
            _endId
            endNode {
                uri
            }
        }
    }
}
```

Create graph node:

```
mutation CreateGraphNode {
    createGraphNode(label: "label") {
        _id,
        uri,
    }
}
```

Delete graph node:

```
mutation DeleteGraphNode {
    deleteGraphNode(uri: "http://example.com/entity/15")
}
```

Create relationship:

```
mutation CreateGraphNode {
    createNodeRelationship(startNodeUri: "http://example.com/entity/13", endNodeUri: "http://example.com/entity/3", relationshipType: "rel1") {
        _id
    }
}
```

## Issues

- Neighbor count is wrong (I'm counting all relations).
- Delete graph node always returns zero.
- Logging could be improved (seems like GraphQL tracing is not working)
- Use of `unwrap()` in codebase
- Injection vulnerability

## Improvements

- Metrics (hooked up to Prometheus)
- Authentication
- Load shedding
- Pagination, perhaps
- Unit tests
