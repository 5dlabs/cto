//! Test workload manifests for validation.
//!
//! This module contains Kubernetes manifests for test workloads that
//! the AI agent will deploy to validate cluster functionality.

// These constants are reference material embedded in the AI agent prompt
#![allow(dead_code)]

/// Basic pod test - validates pod scheduling and execution.
pub const TEST_POD: &str = r#"
apiVersion: v1
kind: Pod
metadata:
  name: validation-test-pod
  namespace: default
  labels:
    app: validation-test
spec:
  containers:
  - name: test
    image: nginx:alpine
    ports:
    - containerPort: 80
    resources:
      limits:
        memory: "64Mi"
        cpu: "100m"
      requests:
        memory: "32Mi"
        cpu: "50m"
  restartPolicy: Never
  securityContext:
    runAsNonRoot: true
    runAsUser: 65534
    seccompProfile:
      type: RuntimeDefault
"#;

/// Storage test - validates PVC binding and volume mounts.
pub const TEST_STORAGE: &str = r#"
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: validation-test-pvc
  namespace: default
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 100Mi
---
apiVersion: v1
kind: Pod
metadata:
  name: validation-test-storage
  namespace: default
  labels:
    app: validation-test
spec:
  containers:
  - name: test
    image: busybox:1.36
    command: ["sh", "-c", "echo 'validation-test' > /data/test.txt && cat /data/test.txt && sleep 10"]
    volumeMounts:
    - name: data
      mountPath: /data
    resources:
      limits:
        memory: "32Mi"
        cpu: "50m"
    securityContext:
      allowPrivilegeEscalation: false
      capabilities:
        drop:
        - ALL
  volumes:
  - name: data
    persistentVolumeClaim:
      claimName: validation-test-pvc
  restartPolicy: Never
  securityContext:
    runAsNonRoot: true
    runAsUser: 65534
    fsGroup: 65534
    seccompProfile:
      type: RuntimeDefault
"#;

/// Network test - validates cross-node pod communication.
pub const TEST_NETWORK_SERVER: &str = r#"
apiVersion: v1
kind: Pod
metadata:
  name: validation-test-server
  namespace: default
  labels:
    app: validation-test-server
spec:
  containers:
  - name: server
    image: nginx:alpine
    ports:
    - containerPort: 80
    resources:
      limits:
        memory: "64Mi"
        cpu: "100m"
  restartPolicy: Never
  securityContext:
    runAsNonRoot: true
    runAsUser: 65534
    seccompProfile:
      type: RuntimeDefault
---
apiVersion: v1
kind: Service
metadata:
  name: validation-test-server
  namespace: default
spec:
  selector:
    app: validation-test-server
  ports:
  - port: 80
    targetPort: 80
"#;

/// Network client - tests service discovery and connectivity.
pub const TEST_NETWORK_CLIENT: &str = r#"
apiVersion: v1
kind: Pod
metadata:
  name: validation-test-client
  namespace: default
  labels:
    app: validation-test-client
spec:
  containers:
  - name: client
    image: busybox:1.36
    command: ["sh", "-c", "wget -q -O- http://validation-test-server.default.svc.cluster.local:80 && echo 'SUCCESS'"]
    resources:
      limits:
        memory: "32Mi"
        cpu: "50m"
    securityContext:
      allowPrivilegeEscalation: false
      capabilities:
        drop:
        - ALL
  restartPolicy: Never
  securityContext:
    runAsNonRoot: true
    runAsUser: 65534
    seccompProfile:
      type: RuntimeDefault
"#;

/// Cleanup manifest - removes all test resources.
pub const CLEANUP_SCRIPT: &str = r"
kubectl delete pod validation-test-pod validation-test-storage validation-test-server validation-test-client -n default --ignore-not-found --grace-period=0 --force
kubectl delete pvc validation-test-pvc -n default --ignore-not-found
kubectl delete service validation-test-server -n default --ignore-not-found
";
