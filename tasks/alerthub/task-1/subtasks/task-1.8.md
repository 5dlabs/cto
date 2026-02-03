# Subtask 1.8: Deploy Ingress and Load Balancer

## Parent Task
Task 1

## Agent
ingress-deployer

## Parallelizable
Yes

## Description
Configure NGINX Ingress controller and external load balancers for traffic routing.

## Details
- Install NGINX Ingress Controller
- Configure TLS termination with cert-manager
- Set up external LoadBalancer services
- Configure global rate limiting
- Implement A/B testing routing rules
- Set up URL rewriting rules

## Deliverables
- `ingress-controller.yaml` - NGINX deployment
- `cert-manager.yaml` - TLS certificate management
- `ingress-routes.yaml` - Ingress configurations
- `loadbalancer-services.yaml` - External services

## Acceptance Criteria
- [ ] Ingress controller is Running
- [ ] TLS certificates are issued and valid
- [ ] External traffic routes to services
- [ ] Rate limiting is enforced

## Testing Strategy
- curl endpoints with valid TLS
- Verify rate limiting blocks excess requests
- Check service endpoints respond
