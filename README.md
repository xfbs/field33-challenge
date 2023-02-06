# Field33 Challenge

GraphQL backend for abstract graph editor.

## Statistics

Execution time: ~7 hours.

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

## Issues

Neighbor count is wrong (I'm counting all relations).
