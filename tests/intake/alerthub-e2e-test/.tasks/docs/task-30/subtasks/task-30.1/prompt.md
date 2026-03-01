# Subtask 30.1: Deploy Core Microservices to Kubernetes

## Parent Task
Task 30

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Deploy all application microservices to Kubernetes cluster with proper resource limits, health checks, and HPA configuration

## Dependencies
None

## Implementation Details
Create Kubernetes manifests (Deployments, Services, ConfigMaps, Secrets) for all microservices. Configure resource requests/limits, liveness/readiness probes, and Horizontal Pod Autoscaler policies. Apply proper labels and annotations for service discovery and monitoring.

## Test Strategy
Verify all pods are running, health checks passing, and services are accessible

---
*Project: alerthub*
