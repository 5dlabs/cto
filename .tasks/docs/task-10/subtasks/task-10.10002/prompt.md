Implement subtask 10002: Scale CNPG PostgreSQL cluster to 2 instances with WAL archival to R2

## Objective
Update the CloudNativePG Cluster CR to instances: 2 (1 primary + 1 replica). Enable WAL archival to the sigma1-wal-archive R2 bucket using the barmanObjectStore configuration. Verify streaming replication is active and replication lag is within acceptable bounds.

## Steps
Edit the CNPG Cluster CR in helm/sigma1/templates/postgres-cluster.yaml: `spec.instances: 2`. Add `spec.backup.barmanObjectStore: { destinationPath: 's3://sigma1-wal-archive/', endpointURL: 'https://<account>.r2.cloudflarestorage.com', s3Credentials: { accessKeyId: { name: sigma1-r2-secret, key: ACCESS_KEY_ID }, secretAccessKey: { name: sigma1-r2-secret, key: SECRET_ACCESS_KEY } }, wal: { compression: gzip } }`. Apply via helm upgrade. Run `kubectl get cluster sigma1-postgres -n databases` — confirm instances=2, status=Ready. Run `kubectl exec -n databases sigma1-postgres-1 -- psql -c 'SELECT * FROM pg_stat_replication;'` to confirm replica connected.

## Validation
`kubectl get cluster sigma1-postgres -n databases` shows `instances: 2` and `readyInstances: 2`. `kubectl exec` pg_stat_replication query returns 1 row showing streaming replica. R2 bucket sigma1-wal-archive receives WAL segment files within 60s of a test write (check R2 console or `aws s3 ls s3://sigma1-wal-archive/` with R2-compatible CLI).