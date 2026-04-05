Implement subtask 9002: Configure CloudNative-PG automated backups to R2

## Objective
Set up automated barman backups from CloudNative-PG to the Cloudflare R2 bucket on a 6-hour schedule, and verify backup/restore functionality.

## Steps
1. Create a Kubernetes Secret with R2 credentials (access key ID and secret access key) for barman:
   - `kubectl create secret generic sigma1-postgres-backup-creds --from-literal=ACCESS_KEY_ID=<key> --from-literal=ACCESS_SECRET_KEY=<secret>`
2. Update the `sigma1-postgres` Cluster CR to add backup configuration:
   - `spec.backup.barmanObjectStore.destinationPath: s3://sigma1-db-backups/`
   - `spec.backup.barmanObjectStore.endpointURL: https://<account-id>.r2.cloudflarestorage.com`
   - `spec.backup.barmanObjectStore.s3Credentials.accessKeyId` referencing the secret
   - `spec.backup.barmanObjectStore.s3Credentials.secretAccessKey` referencing the secret
   - `spec.backup.barmanObjectStore.wal.compression: gzip`
3. Create a ScheduledBackup CR:
   - `apiVersion: postgresql.cnpg.io/v1`, `kind: ScheduledBackup`
   - `spec.schedule: '0 */6 * * *'`
   - `spec.cluster.name: sigma1-postgres`
   - `spec.backupOwnerReference: self`
4. Trigger a manual backup to verify: `kubectl cnpg backup sigma1-postgres`
5. Verify backup object appears in R2 bucket.
6. Test restore by creating a recovery cluster CR that restores from the backup and verifying data integrity.

## Validation
Trigger a manual backup via `kubectl cnpg backup sigma1-postgres`, verify it completes successfully and an object appears in the R2 bucket. Create a temporary recovery Cluster CR pointing to the backup, verify it restores and data is intact, then delete the recovery cluster.