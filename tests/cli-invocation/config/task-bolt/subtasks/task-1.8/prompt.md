# Configure Network Policies

<task>
<agent>network-agent</agent>
<objective>Configure network policies for infrastructure components</objective>

<context>
Infrastructure components are deployed. Your job is to configure network policies to control traffic flow between services and ensure proper isolation.
</context>

<requirements>
- Create NetworkPolicy resources for each namespace
- Configure ingress rules for external access
- Configure egress rules for outbound traffic
- Set up service mesh policies if applicable
- Ensure database services are only accessible from authorized pods
</requirements>

<deliverables>
- `networkpolicy-databases.yaml` - Database network policies
- `networkpolicy-messaging.yaml` - Messaging network policies
- `ingress.yaml` - Ingress configuration (if needed)
</deliverables>

<acceptance_criteria>
- [ ] Network policies are applied
- [ ] Database services are properly isolated
- [ ] Only authorized pods can access databases
- [ ] Egress is restricted appropriately
</acceptance_criteria>
</task>
