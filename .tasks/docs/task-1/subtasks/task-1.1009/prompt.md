Implement subtask 1009: Create Cilium NetworkPolicy CRs

## Objective
Deploy Cilium CiliumNetworkPolicy custom resources to enforce network segmentation: allow sigma1 pods to reach PostgreSQL and Valkey, allow Morgan to reach all backend services, deny cross-namespace traffic by default.

## Steps
1. Create `sigma1-default-deny.yaml` — a CiliumNetworkPolicy that denies all ingress from outside the sigma1 namespace:
   ```yaml
   apiVersion: cilium.io/v2
   kind: CiliumNetworkPolicy
   metadata:
     name: sigma1-default-deny
     namespace: sigma1
   spec:
     endpointSelector: {}
     ingressDeny:
       - fromEndpoints:
           - matchExpressions:
               - key: k8s:io.kubernetes.pod.namespace
                 operator: NotIn
                 values: [sigma1]
   ```
2. Create `sigma1-allow-datastore.yaml` — allow all sigma1 pods to reach PostgreSQL (port 5432) and Valkey (port 6379):
   ```yaml
   spec:
     endpointSelector:
       matchLabels:
         io.kubernetes.pod.namespace: sigma1
     egress:
       - toEndpoints:
           - matchLabels:
               cnpg.io/cluster: sigma1-postgres
         toPorts:
           - ports:
               - port: "5432"
       - toEndpoints:
           - matchLabels:
               app: sigma1-valkey
         toPorts:
           - ports:
               - port: "6379"
   ```
3. Create `sigma1-allow-morgan.yaml` — allow the Morgan agent pod (label: `app: morgan`) to reach all backend services in sigma1 namespace on their service ports.
4. Apply all CiliumNetworkPolicy manifests.
5. Ensure DNS egress is also allowed (port 53 to kube-dns) so services can resolve names.

## Validation
`kubectl get ciliumnetworkpolicies -n sigma1` shows all 3 policies. Test with a pod in a different namespace: `kubectl run --rm -it --image=busybox test-deny -n default -- wget -T5 -O- sigma1-postgres-rw.sigma1.svc.cluster.local:5432` should be blocked. A pod within sigma1 namespace with appropriate labels should successfully connect to PostgreSQL and Valkey. Cilium hubble observe shows deny verdicts for cross-namespace traffic.