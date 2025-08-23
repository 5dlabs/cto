# PostgreSQL with pgvector Extension

Custom PostgreSQL image that combines:


- **Zalando Spilo PostgreSQL 16** - Operator-compatible with cluster management features


- **pgvector extension** - Vector similarity search and high-dimensional vector support



## Features



- ✅ **High-dimensional vector support** - Supports 3072-dimension vectors for text-embedding-3-large compatibility


- ✅ **Operator compatibility** - Works with Zalando postgres operator and Spilo clusters


- ✅ **Cluster management** - Full support for HA, streaming replication, and automated failover


- ✅ **Production ready** - Based on Zalando's production-tested Spilo image



## Usage



### Docker




```bash
docker run --name postgres-vector \


  -e POSTGRESQL_PASSWORD=mypassword \
  -p 5432:5432 \
  ghcr.io/5dlabs/postgresql:latest








```

### Enable pgvector extension




```sql
CREATE EXTENSION IF NOT EXISTS vector;



-- Create a table with vector column
CREATE TABLE embeddings (
  id serial PRIMARY KEY,
  content text,
  embedding vector(3072)  -- Supports up to 3072 dimensions
);



-- Insert vectors
INSERT INTO embeddings (content, embedding)
VALUES ('sample text', '[0.1,0.2,0.3,...]');



-- Vector similarity search
SELECT content, embedding <-> '[0.1,0.2,0.3,...]' AS distance
FROM embeddings
ORDER BY embedding <-> '[0.1,0.2,0.3,...]'
LIMIT 10;








```

## Kubernetes

Used by the `vector-postgres` PostgreSQL cluster in the `databases` namespace for AI/ML workloads requiring vector similarity search.



## Architecture

This image extends the official Zalando Spilo image with pgvector:


- **ghcr.io/zalando/spilo-16** - Provides PostgreSQL 16 with operator compatibility


- **postgresql-16-pgvector** - Installed from PostgreSQL APT repository

This approach maintains full compatibility with the Zalando postgres operator while adding vector search capabilities.
