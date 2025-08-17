# Custom PostgreSQL image with pgvector for Zalando postgres operator (Spilo) compatibility  
# This image extends the official Spilo image with pgvector extension
# Supports high-dimensional vectors (3072-dim) for text-embedding-3-large compatibility

FROM ghcr.io/zalando/spilo-16:3.3-p1

# Switch to root to install pgvector
USER root

# Install pgvector extension from PostgreSQL APT repository
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    postgresql-16-pgvector \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Verify pgvector installation
RUN ls -la /usr/lib/postgresql/16/lib/*vector* && \
    ls -la /usr/share/postgresql/16/extension/vector*

# Switch back to postgres user (Spilo default)
USER postgres