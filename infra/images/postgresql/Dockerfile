# Custom PostgreSQL image with pgvector for Bitnami chart compatibility  
# This image bridges the gap between pgvector/pgvector and Bitnami's directory structure
# Supports high-dimensional vectors (3072-dim) for text-embedding-3-large compatibility

FROM pgvector/pgvector:pg16 AS pgvector-source
FROM bitnami/postgresql:16.3.0-debian-12-r19

# Copy pgvector extension files to Bitnami's expected locations
COPY --from=pgvector-source /usr/lib/postgresql/16/lib/vector.so /opt/bitnami/postgresql/lib/
COPY --from=pgvector-source /usr/share/postgresql/16/extension/vector* /opt/bitnami/postgresql/share/extension/

# Ensure proper permissions for Bitnami's non-root setup
USER root
RUN chmod +r /opt/bitnami/postgresql/lib/vector.so && \
    chmod +r /opt/bitnami/postgresql/share/extension/vector*

# Switch back to Bitnami's default user
USER 1001
