## GitHub Webhooks to Argo Workflows (via ngrok + Argo Events)



### Overview
- Public entry: ngrok Gateway API (HTTPS listener) on your domain → HTTPRoute → Service (`github-eventsource-svc`)
- Event ingestion: Argo Events `EventSource` (GitHub) → NATS `EventBus`
- Processing: Argo Events `Sensor` → triggers an Argo `Workflow` (or other K8s object)



### Prereqs


- ngrok operator installed with Gateway API


- DNS CNAME set to ngrok CNAME target
- Credential ACL allows `bind:<your-host>`


- Argo Events installed; `EventBus` present

### Wiring steps
1) Gateway/Route
- Listener: `hostname: <your-host>` on port 443
- `HTTPRoute` path `/github/webhook` backend `github-eventsource-svc:12000`

2) EventSource (GitHub)
- Pick scope:
  - org-wide: `organizations: [your-org]`
  - or repos: `repositories: [{ owner: your-org, names: [repo1, repo2] }]`
- Set webhook: `endpoint: /github/webhook`, `port: "12000"`, `method: POST`
- Optional: `webhookSecret` (strong secret) to enforce signature validation
- Select events (recommended to narrow):
  - `events: ["issues","pull_request","issue_comment","workflow_run"]`

3) Sensor → Workflow


- Define dependencies filtered by event type/action and map payload fields to Workflow parameters
- RBAC: ServiceAccount for the Sensor to create Workflows



### Examples

#### Issues: opened/closed
- EventSource:




```yaml
spec:
  github:
    main:
      organizations: [ your-org ]
      webhook:
        endpoint: /github/webhook
        port: "12000"
        method: POST
      events: ["issues"]








```
- Sensor (only on opened/closed):




```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: issues-sensor
  namespace: argo
spec:
  eventBusName: default
  template:
    serviceAccountName: argo-events-sa
  dependencies:
    - name: issues
      eventSourceName: github
      eventName: main
      filters:
        data:
          - path: body.action
            type: string
            value:


              - opened


              - closed
  triggers:
    - template:
        name: wf-issues
        argoWorkflow:
          operation: submit
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              metadata:
                generateName: wf-issues-
                namespace: argo
              spec:
                entrypoint: main
                templates:
                  - name: main
                    container:
                      image: alpine:3.20
                      command: [sh, -c]
                      args: ["echo Issue {{workflow.parameters.action}} #{{workflow.parameters.number}} in {{workflow.parameters.repo}}"]
                arguments:
                  parameters:
                    - name: action
                      value: "{{inputs.parameters.action}}"
                    - name: number
                      value: "{{inputs.parameters.number}}"
                    - name: repo
                      value: "{{inputs.parameters.repo}}"
          parameters:
            - src:
                dependencyName: issues
                dataKey: body.action
              dest: spec.arguments.parameters.0.value
            - src:
                dependencyName: issues
                dataKey: body.issue.number
              dest: spec.arguments.parameters.1.value
            - src:
                dependencyName: issues
                dataKey: body.repository.full_name
              dest: spec.arguments.parameters.2.value








```

#### PR created (opened)


- EventSource includes `pull_request`
- Sensor filter: `body.action == opened`


- Map `body.pull_request.number`, `body.repository.full_name`

#### Comment added to PR


- Subscribe to `issue_comment`
- Filter: `body.issue.pull_request != null` and `body.action == created`


  - Use an expression filter (or switch to `jsonPath` with a small transformer step)



#### Workflow failed


- Subscribe to `workflow_run`
- Filter: `body.action == completed` and `body.workflow_run.conclusion == failure`



#### Pull request merged
- Event: `pull_request`
- Filter: `body.action == closed` and `body.pull_request.merged == true`



### RBAC




```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: argo-events-workflow-executor
  namespace: argo
rules:
- apiGroups: ["argoproj.io"]
  resources: ["workflows"]
  verbs: ["create"]


---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: argo-events-workflow-executor
  namespace: argo
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: argo-events-workflow-executor
subjects:
- kind: ServiceAccount
  name: argo-events-sa
  namespace: argo








```



### Tips
- Keep `events:` tight to reduce load/noise.


- Use `application/json` in GitHub webhooks; set a `webhookSecret` and confirm `X-Hub-Signature-256`.


- Test with a signed curl using the same secret before toggling in GitHub.


- For multiple routes/hosts, add listeners to the Gateway and corresponding DNS CNAMEs.

### Promotion


- Move manifests from `infra/gitops/resources/staging/...` to `infra/gitops/resources/...` and add an Application under `infra/gitops/applications/` to follow the existing app-of-apps pattern.


- Keep ExternalSecret for the webhook secret and avoid committing secret data.
