.PHONY: help install uninstall status test clean

# Variables
NAMESPACE := storage
KUSTOMIZE := kubectl apply -k
KUBECTL := kubectl

help:
	@echo "SeaweedFS Kubernetes Deployment"
	@echo ""
	@echo "Usage:"
	@echo "  make install       - Deploy SeaweedFS cluster"
	@echo "  make uninstall     - Remove SeaweedFS cluster"
	@echo "  make status        - Check deployment status"
	@echo "  make test          - Run S3 API tests"
	@echo "  make logs-master   - View master logs"
	@echo "  make logs-volume   - View volume logs"
	@echo "  make logs-filer    - View filer logs"
	@echo "  make logs-s3       - View S3 gateway logs"
	@echo "  make shell         - Open shell in master pod"
	@echo "  make clean         - Clean up test resources"

install:
	@echo "Deploying SeaweedFS..."
	$(KUSTOMIZE) .
	@echo "Waiting for pods to be ready..."
	$(KUBECTL) wait --for=condition=ready pod -l app.kubernetes.io/name=seaweedfs -n $(NAMESPACE) --timeout=300s

uninstall:
	@echo "Removing SeaweedFS..."
	$(KUBECTL) delete -k .
	@echo "Waiting for pods to terminate..."
	$(KUBECTL) wait --for=delete pod -l app.kubernetes.io/name=seaweedfs -n $(NAMESPACE) --timeout=120s || true

status:
	@echo "=== Namespace ==="
	$(KUBECTL) get namespace $(NAMESPACE)
	@echo ""
	@echo "=== Pods ==="
	$(KUBECTL) get pods -n $(NAMESPACE) -l app.kubernetes.io/name=seaweedfs
	@echo ""
	@echo "=== Services ==="
	$(KUBECTL) get svc -n $(NAMESPACE)
	@echo ""
	@echo "=== PVCs ==="
	$(KUBECTL) get pvc -n $(NAMESPACE)
	@echo ""
	@echo "=== StatefulSets ==="
	$(KUBECTL) get statefulset -n $(NAMESPACE)

test:
	@echo "Running S3 API test..."
	$(KUBECTL) apply -f examples/s3-test.yaml
	@echo "Waiting for test to complete..."
	$(KUBECTL) wait --for=condition=complete job/seaweedfs-s3-test -n $(NAMESPACE) --timeout=120s
	@echo ""
	@echo "Test Results:"
	$(KUBECTL) logs job/seaweedfs-s3-test -n $(NAMESPACE)
	@echo ""
	@echo "Cleaning up test job..."
	$(KUBECTL) delete job seaweedfs-s3-test -n $(NAMESPACE)

logs-master:
	$(KUBECTL) logs -n $(NAMESPACE) -l app.kubernetes.io/component=master --tail=100 -f

logs-volume:
	$(KUBECTL) logs -n $(NAMESPACE) -l app.kubernetes.io/component=volume --tail=100 -f

logs-filer:
	$(KUBECTL) logs -n $(NAMESPACE) -l app.kubernetes.io/component=filer --tail=100 -f

logs-s3:
	$(KUBECTL) logs -n $(NAMESPACE) -l app.kubernetes.io/component=s3 --tail=100 -f

shell:
	$(KUBECTL) exec -it -n $(NAMESPACE) seaweedfs-master-0 -- weed shell -master=localhost:9333

clean:
	@echo "Cleaning up test resources..."
	$(KUBECTL) delete job -n $(NAMESPACE) -l app.kubernetes.io/name=seaweedfs-test || true
	$(KUBECTL) delete pod -n $(NAMESPACE) seaweedfs-client || true

cluster-status:
	$(KUBECTL) exec -n $(NAMESPACE) seaweedfs-master-0 -- weed shell -master=localhost:9333 -command "cluster.status"

volume-list:
	$(KUBECTL) exec -n $(NAMESPACE) seaweedfs-master-0 -- weed shell -master=localhost:9333 -command "volume.list"

check-health:
	@echo "=== Master Health ==="
	$(KUBECTL) exec -n $(NAMESPACE) seaweedfs-master-0 -- wget -q -O- http://localhost:9333/cluster/status
	@echo ""
	@echo "=== Volume Health ==="
	$(KUBECTL) exec -n $(NAMESPACE) seaweedfs-volume-0 -- wget -q -O- http://localhost:8080/status
	@echo ""
	@echo "=== Filer Health ==="
	$(KUBECTL) exec -n $(NAMESPACE) seaweedfs-filer-0 -- wget -q -O- http://localhost:8888/

scale-volumes:
	@read -p "Enter number of volume replicas: " replicas; \
	$(KUBECTL) scale statefulset seaweedfs-volume -n $(NAMESPACE) --replicas=$$replicas

scale-filers:
	@read -p "Enter number of filer replicas: " replicas; \
	$(KUBECTL) scale statefulset seaweedfs-filer -n $(NAMESPACE) --replicas=$$replicas
