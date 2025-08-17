# PostgreSQL with pgvector Extension

Custom PostgreSQL image that combines:
- **Bitnami PostgreSQL 16.3.0** - Production-ready with security optimizations
- **pgvector extension** - Vector similarity search and high-dimensional vector support

## Features

- ✅ **High-dimensional vector support** - Supports 3072-dimension vectors for text-embedding-3-large compatibility
- ✅ **Bitnami compatibility** - Works with Bitnami PostgreSQL Helm charts and operators
- ✅ **Security optimized** - Runs as non-root user (1001) with proper permissions
- ✅ **Production ready** - Based on Bitnami's production-hardened PostgreSQL image

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

This image solves the compatibility issue between:
- **pgvector/pgvector** - Provides the pgvector extension
- **bitnami/postgresql** - Provides production-ready PostgreSQL with security hardening

The multi-stage build copies pgvector extension files to Bitnami's expected directory structure while maintaining proper permissions for non-root operation.
