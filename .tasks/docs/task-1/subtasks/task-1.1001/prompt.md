Implement subtask 1001: Create core Kubernetes namespaces

## Objective
Create the 'databases' and 'sigma1' Kubernetes namespaces to logically separate infrastructure components and application services.

## Steps
Execute `kubectl create namespace databases` and `kubectl create namespace sigma1` commands. Verify creation using `kubectl get ns`.

## Validation
Verify both 'databases' and 'sigma1' namespaces are listed when running `kubectl get namespaces`.