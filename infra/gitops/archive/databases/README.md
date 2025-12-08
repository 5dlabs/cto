# Database Operators and Resources

This directory contains database operator configurations and custom resources for managing PostgreSQL, Redis, and QuestDB clusters in Kubernetes.

## PostgreSQL Operator (Zalando)

The Zalando PostgreSQL operator manages PostgreSQL clusters with high availability, automatic failover, and backup capabilities.



### Features
- **High Availability**: Multi-replica PostgreSQL clusters with streaming replication via Patroni
- **Automatic Failover**: Built-in leader election and automatic promotion
- **Connection Pooling**: Integrated PgBouncer support
- **Backup & Recovery**: Point-in-time recovery with pg_basebackup/WAL-E
- **Rolling Updates**: Zero-downtime updates and version upgrades
- **Resource Management**: CPU/memory limits and requests per cluster

### Creating a PostgreSQL Cluster

Example minimal cluster:




```yaml
apiVersion: acid.zalan.do/v1
kind: postgresql
metadata:
  name: my-postgres-cluster
spec:
  teamId: "platform"
  volume:
    size: 10Gi
  numberOfInstances: 2
  users:
    myapp:


    - superuser


    - createdb
  databases:
    mydb: myapp
  postgresql:
    version: "17"








```

### Useful Commands




```bash


# List PostgreSQL clusters
kubectl get postgresql -A



# Get cluster details
kubectl describe postgresql my-postgres-cluster



# Access master pod
kubectl exec -it my-postgres-cluster-0 -- psql -U postgres



# Check cluster status
kubectl get pods -l cluster-name=my-postgres-cluster








```



## Redis Operator (Spotahome)

The Spotahome Redis operator manages Redis failover clusters with Sentinel for high availability.



### Features
- **High Availability**: Redis with Sentinel-based automatic failover
- **Automatic Failover**: Master election via Redis Sentinel
- **Data Persistence**: Optional RDB/AOF persistence
- **Security**: Password authentication and TLS support
- **Monitoring**: Prometheus metrics export
- **Resource Management**: Configurable resources per component

### Creating a Redis Failover Cluster

Example minimal cluster:




```yaml
apiVersion: databases.spotahome.com/v1
kind: RedisFailover
metadata:
  name: my-redis-cluster
spec:
  sentinel:
    replicas: 3
    resources:
      requests:
        cpu: 100m
        memory: 100Mi
      limits:
        cpu: 500m
        memory: 200Mi
  redis:
    replicas: 3
    resources:
      requests:
        cpu: 100m
        memory: 200Mi
      limits:
        cpu: 1000m
        memory: 1Gi
    storage:
      persistentVolumeClaim:
        metadata:
          name: redis-data
        spec:
          accessModes:


            - ReadWriteOnce
          resources:
            requests:
              storage: 10Gi








```

### Useful Commands




```bash


# List Redis failover clusters
kubectl get redisfailover -A



# Get cluster details
kubectl describe redisfailover my-redis-cluster



# Access Redis master
kubectl exec -it rfr-my-redis-cluster-0 -- redis-cli

# Check Sentinel status
kubectl exec -it rfs-my-redis-cluster-0 -- redis-cli -p 26379 sentinel masters








```



## QuestDB Operator

The QuestDB operator manages high-performance time-series database instances optimized for real-time analytics and metrics storage.



### Features
- **High-Performance Ingestion**: Millions of data points per second
- **SQL Interface**: PostgreSQL wire protocol compatibility
- **Time-Series Optimized**: Columnar storage with time-based partitioning
- **Multiple Protocols**: HTTP REST API, PostgreSQL wire, InfluxDB Line Protocol
- **Out-of-Order Ingestion**: Handles late-arriving data efficiently
- **SIMD Optimizations**: Vectorized query execution
- **Zero-GC**: Runs on custom Java runtime with minimal garbage collection

### Creating a QuestDB Instance

Example minimal instance:




```yaml
apiVersion: crd.questdb.io/v1beta1
kind: QuestDB
metadata:
  name: my-questdb
spec:
  image:
    repository: questdb/questdb
    tag: "7.3.10"
  volume:
    size: 50Gi
  resources:
    requests:
      cpu: 500m
      memory: 2Gi
    limits:
      cpu: 2000m
      memory: 8Gi
  config:
    http.enabled: "true"
    pg.enabled: "true"
    line.tcp.enabled: "true"








```

### Useful Commands




```bash
# List QuestDB instances
kubectl get questdb -A

# Get instance details
kubectl describe questdb my-questdb

# Access QuestDB console
kubectl port-forward svc/my-questdb-http 9000:9000
# Then browse to http://localhost:9000



# Execute SQL via REST API
curl -G http://localhost:9000/exec \


  --data-urlencode "query=SELECT * FROM metrics LIMIT 10"

# Connect via PostgreSQL protocol
psql -h localhost -p 8812 -U admin -d qdb








```

### Data Ingestion Methods

1. **InfluxDB Line Protocol (ILP)** - Fastest for high-volume ingestion:




```bash
echo "sensors,location=lab1 temp=23.5,humidity=45.2 $(date +%s%N)" | \
  nc questdb-host 9009








```

2. **PostgreSQL Wire Protocol** - Standard SQL interface:




```sql
INSERT INTO metrics VALUES(now(), 'cpu', 45.2);








```

3. **HTTP REST API** - For application integration:




```bash
curl -X POST http://questdb:9000/exec \


  -d "query=INSERT INTO metrics VALUES(now(), 'memory', 78.5)"








```



## Best Practices



### PostgreSQL


1. **Always use at least 2 instances** for high availability


2. **Configure appropriate resource limits** based on workload


3. **Enable monitoring** via the built-in metrics exporter


4. **Use connection pooling** for applications with many connections


5. **Regular backups** - configure WAL archiving for PITR


6. **Version upgrades** - test in non-production first



### Redis


1. **Use odd number of Sentinels** (3 or 5) for proper quorum


2. **Configure persistence** based on data criticality


3. **Set memory policies** (`maxmemory-policy`) appropriately


4. **Monitor memory usage** to prevent OOM situations


5. **Use AUTH** for production deployments


6. **Regular backups** if data persistence is critical



### QuestDB


1. **Partition by time** for optimal query performance


2. **Use ILP for high-volume ingestion** instead of SQL inserts


3. **Configure appropriate heap size** (typically 50-75% of container memory)


4. **Enable WAL for durability** in production environments


5. **Use SAMPLE BY** for time-based aggregations


6. **Monitor disk I/O** as time-series workloads are I/O intensive


7. **Set appropriate commit lag** to balance ingestion speed vs durability

## Monitoring

All database operators expose Prometheus metrics:

- **PostgreSQL**: Port 9187 on each PostgreSQL pod
- **Redis**: Port 9121 on Redis pods, port 26379 on Sentinel pods
- **QuestDB**: Port 9003 for built-in metrics endpoint

Configure ServiceMonitors or scrape configs to collect these metrics.

## Troubleshooting



### PostgreSQL Issues




```bash


# Check operator logs
kubectl logs -n postgres-operator deployment/postgres-operator

# Check Patroni status
kubectl exec my-postgres-cluster-0 -- patronictl list



# Check PostgreSQL logs
kubectl logs my-postgres-cluster-0 -c postgres








```



### Redis Issues




```bash


# Check operator logs
kubectl logs -n redis-operator deployment/redisoperator

# Check Sentinel logs
kubectl logs rfs-my-redis-cluster-0



# Check Redis logs
kubectl logs rfr-my-redis-cluster-0








```



### QuestDB Issues




```bash


# Check operator logs
kubectl logs -n questdb-operator deployment/questdb-operator-controller-manager

# Check QuestDB instance logs
kubectl logs questdb-instance-name-0

# Access QuestDB console for diagnostics
kubectl port-forward svc/questdb-instance-http 9000:9000


# Browse to http://localhost:9000



# Check table health
curl -G http://localhost:9000/exec \


  --data-urlencode "query=SHOW TABLES"








```

## Security Considerations

1. **Network Policies**: Implement network policies to restrict database access
2. **Secrets Management**: Use Vault Secrets Operator for credential management
3. **Encryption**: Enable TLS for client connections and replication
4. **RBAC**: Limit operator permissions to necessary namespaces
5. **Audit Logging**: Enable audit logging for compliance requirements

## References

- [Zalando PostgreSQL Operator Documentation](https://postgres-operator.readthedocs.io/)
- [Spotahome Redis Operator Documentation](https://github.com/spotahome/redis-operator/tree/master/docs)
- [QuestDB Operator Documentation](https://github.com/questdb/questdb-operator)
- [PostgreSQL CRD Reference](https://postgres-operator.readthedocs.io/en/latest/reference/cluster_manifest/)
- [RedisFailover CRD Reference](https://github.com/spotahome/redis-operator/blob/master/api/redisfailover/v1/validate.go)
- [QuestDB Documentation](https://questdb.io/docs/)
- [QuestDB SQL Reference](https://questdb.io/docs/reference/sql/overview/)
