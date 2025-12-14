# Custom PostgreSQL image with pgvector for Zalando postgres operator (Spilo) compatibility  
# This image extends the official Spilo image with pgvector extension
# Supports high-dimensional vectors (3072-dim) for text-embedding-3-large compatibility

FROM ghcr.io/zalando/spilo-16:3.3-p3

# Switch to root to install pgvector
USER root

# Install pgvector extension from PostgreSQL APT repository
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    postgresql-16-pgvector \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Verify pgvector installation and 3072-dimension support
RUN ls -la /usr/lib/postgresql/16/lib/*vector* && \
    ls -la /usr/share/postgresql/16/extension/vector* && \
    echo "pgvector installed successfully"

# Test 3072-dimensional vector support during build
RUN export PGDATA=/tmp/pgdata && \
    su postgres -c 'initdb -D /tmp/pgdata && \
    pg_ctl -D /tmp/pgdata -l /tmp/pgdata/logfile start -w && \
    psql -c "CREATE EXTENSION vector;" && \
    psql -c "CREATE TABLE test_3072 (embedding vector(3072));" && \
    psql -c "INSERT INTO test_3072 VALUES (array_fill(0.1, ARRAY[3072])::vector);" && \
    psql -c "SELECT vector_dims(embedding) FROM test_3072;" && \
    pg_ctl -D /tmp/pgdata stop -w' && \
    rm -rf /tmp/pgdata && \
    echo "✅ 3072-dimensional vector support verified!"

# Switch back to postgres user (Spilo default)  
USER postgres