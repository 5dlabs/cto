Implement subtask 9002: Scale Redis to HA multi-replica mode via operator CR

## Objective
Update the Redis operator custom resource to enable high-availability with Sentinel or Redis Cluster mode, ensuring automatic failover and read replica distribution across nodes.

## Steps
1. Edit the Redis operator CR to enable HA mode (e.g., Redis Sentinel with 3 sentinels and 1+ replicas, or Redis Cluster depending on operator).
2. Set `replicas` for Redis data nodes and sentinel nodes.
3. Add pod anti-affinity rules to distribute Redis and Sentinel pods across nodes.
4. Configure persistence (AOF/RDB) with appropriate storage class.
5. Set memory limits and maxmemory-policy for production.
6. Apply the updated CR and wait for all replicas and sentinels to become ready.
7. Verify Sentinel quorum is established and replicas are synchronized.

## Validation
Verify Redis master, replicas, and sentinels are all running on distinct nodes. Confirm replication is active via `INFO replication`. Kill the Redis master pod and verify Sentinel promotes a replica within expected timeframe. Confirm application clients reconnect to new master.