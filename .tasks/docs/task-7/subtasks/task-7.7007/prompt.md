<identity>
You are angie working on subtask 7007 of task 7.
</identity>

<context>
<scope>
Enable the OpenClaw web chat adapter on port 3000, serving the chat API endpoint and the embeddable widget JavaScript, and create the ClusterIP Kubernetes Service.
</scope>
</context>

<implementation_plan>
1. In agents/morgan/agent.yaml, add web chat adapter block:
   adapters:
     web_chat:
       enabled: true
       port: 3000
       cors_origins: ['*']  # tighten to Sigma-1 domain post-launch
       widget_path: /widget.js
       chat_path: /chat
2. The /chat endpoint accepts POST {message: string, session_id?: string} and returns {reply: string, session_id: string}.
3. The /widget.js endpoint returns a self-contained JavaScript snippet that renders a floating chat button and slide-up panel targeting the /chat endpoint.
4. Create k8s/openclaw/morgan-chat-svc.yaml: ClusterIP Service in openclaw namespace, selector: app=morgan, port 3000 → targetPort 3000, name: morgan-chat-svc.
5. Apply service manifest: `kubectl apply -f k8s/openclaw/morgan-chat-svc.yaml`.
</implementation_plan>

<validation>
After pod is running: `kubectl port-forward svc/morgan-chat-svc 3000:3000 -n openclaw`. POST http://localhost:3000/chat with {"message": "hello"} returns JSON with non-empty reply field within 10s. GET http://localhost:3000/widget.js returns JavaScript with Content-Type: application/javascript.
</validation>