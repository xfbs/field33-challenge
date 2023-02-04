DOCKER=docker

database:
	docker run -it --rm --publish=7474:7474 --publish=7687:7687 --env=NEO4J_AUTH=neo4j/localpw --env=NEO4J_ACCEPT_LICENSE_AGREEMENT=yes fld33/field33-backend-takehome-neo4j

test:
	cargo test
