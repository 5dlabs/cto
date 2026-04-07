Implement subtask 1006: Deploy Signal-CLI pod, PVC, and ClusterIP Service in the signal namespace

## Objective
Create a PersistentVolumeClaim, Pod (or Deployment), and ClusterIP Service for Signal-CLI in the signal namespace with a 5Gi RWO volume mounted at the Signal data path.

## Steps
Create sigma1/infra/templates/signal-cli.yaml. PersistentVolumeClaim: name signal-cli-data, namespace signal, accessModes: [ReadWriteOnce], storage: 5Gi, storageClassName: default (or parameterize). Deployment (single replica): name signal-cli, namespace signal, image: signalapp/signal-cli:latest, volumeMounts: [{name: data, mountPath: /home/signal-cli/.local/share/signal-cli}], volumes: [{name: data, persistentVolumeClaim: {claimName: signal-cli-data}}]. Resources: limits 256Mi/250m for dev. ClusterIP Service: name signal-cli-svc, namespace signal, port 7583 targeting container port 7583 (JSON-RPC mode). Add a NOTE in a NOTES.txt or README: 'Manual step required before first use: kubectl exec -n signal deploy/signal-cli -- signal-cli -u +<PHONE> register && signal-cli -u +<PHONE> verify <CODE>'. Ensure the pod starts in daemon/JSON-RPC mode: args: ['-u', '$(SIGNAL_NUMBER)', 'daemon', '--tcp', '0.0.0.0:7583'] with SIGNAL_NUMBER as an env var (placeholder secret or configmap key).

## Validation
kubectl get pvc signal-cli-data -n signal shows STATUS=Bound. kubectl get pods -n signal shows signal-cli pod in Running state. kubectl get svc signal-cli-svc -n signal shows ClusterIP with port 7583. curl from within cluster: kubectl run test --rm -it --image=curlimages/curl --restart=Never -- curl -s http://signal-cli-svc.signal.svc.cluster.local:7583 returns a response (even an error JSON is acceptable, confirming the port is reachable).