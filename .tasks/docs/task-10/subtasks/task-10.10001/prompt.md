<identity>
You are bolt working on subtask 10001 of task 10.
</identity>

<context>
<scope>
Update all service Deployment manifests in helm/sigma1/ to set replicas: 2 (except Morgan which stays at 1). Create HorizontalPodAutoscaler manifests for equipment-catalog (min=2, max=8, CPU=70%), rms (min=2, max=6, CPU=70%), finance (min=2, max=4, CPU=70%), customer-vetting (min=2, max=4, CPU=70%), and social-engine (min=2, max=4, CPU=70%). Apply and verify.
</scope>
</context>

<implementation_plan>
For each service Deployment YAML under helm/sigma1/templates/: set `spec.replicas: 2`. For Morgan deployment set `spec.replicas: 1` and add annotation `sigma1/replica-reason: signal-cli-stateful`. Create helm/sigma1/templates/hpa-equipment-catalog.yaml: `apiVersion: autoscaling/v2, kind: HorizontalPodAutoscaler, spec.minReplicas: 2, spec.maxReplicas: 8, metrics[0].resource.name: cpu, metrics[0].resource.target.averageUtilization: 70`. Repeat for rms (max=6), finance (max=4), customer-vetting (max=4), social-engine (max=4). Run `helm upgrade sigma1 ./helm/sigma1 -n sigma1`. Verify with `kubectl get deploy,hpa -n sigma1`.
</implementation_plan>

<validation>
`kubectl get pods -n sigma1` shows READY=2/2 for all services except Morgan (1/1). `kubectl get hpa -n sigma1` shows 5 HPA objects each with MINPODS=2 and correct MAXPODS. Describe each HPA and confirm CPU target=70%.
</validation>