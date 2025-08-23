# Argo Events Reference Documentation

This directory contains official Argo Events examples copied directly from the [argoproj/argo-events](https://github.com/argoproj/argo-events) repository. These serve as authoritative reference for proper Argo Events sensor and trigger configurations.



## 📋 Purpose

When implementing Argo Events sensors/triggers, **ALWAYS** reference these official examples to ensure you're using supported syntax and operations. These examples prevent common mistakes like using unsupported operations or incorrect template variable usage.

## 📂 Reference Files

### Core Sensor Patterns


- **`github.yaml`** - Official GitHub webhook sensor with proper parameter extraction and workflow creation


- **`complete-trigger-parameterization.yaml`** - Dynamic parameter extraction from event payloads


- **`trigger-with-template.yaml`** - Template variable usage and dataTemplate patterns

### Workflow Integration


- **`special-workflow-trigger.yaml`** - ArgoWorkflow operations (submit, resume) with proper syntax


- **`trigger-standard-k8s-resource.yaml`** - Standard Kubernetes resource creation patterns

### Event Sources


- **`github-eventsource.yaml`** - GitHub EventSource configuration for webhook processing

## ⚠️ Critical Lessons Learned

### ❌ **Operations NOT Supported by Argo Events:**
- `operation: delete` ❌
- `operation: patch` ❌
- `operation: update` ❌

### ✅ **Supported Operations:**
- `operation: create` ✅ (k8s resources)
- `operation: submit` ✅ (Argo Workflows)
- `operation: resume` ✅ (Argo Workflows)
- `operation: append` ✅ (parameter modification)
- `operation: prepend` ✅ (parameter modification)

### ❌ **Template Variables NOT Supported in:**


- `labelSelector` fields ❌


- Static YAML structure fields ❌

### ✅ **Template Variables Supported in:**


- `parameters[].dest` values ✅


- `dataTemplate` expressions ✅


- `metadata.name` and `metadata.generateName` ✅


- `spec.arguments.parameters[].value` ✅

## 🎯 Usage Guidelines

1. **Before implementing any Argo Events sensor/trigger:**


   - Review these examples first


   - Match your pattern to an existing example


   - Use only supported operations and syntax

2. **For GitHub webhook sensors:**


   - Use `github.yaml` as the primary reference


   - Follow parameter extraction patterns from `complete-trigger-parameterization.yaml`

3. **For workflow operations:**


   - Use `special-workflow-trigger.yaml` for submit/resume operations
   - For `submit`: parameterize `metadata.name`/`generateName` and spec args
   - For `resume`: pass the existing workflow name via `args` (equivalent to `argo resume <name>`); avoid dynamic `labelSelector`

4. **For resource deletion/cleanup:**
   - Create cleanup workflows with `argoWorkflow.operation: submit`


   - Use workflow scripts with `kubectl delete` commands
   - **DO NOT** use `k8s.operation: delete` (unsupported)

## 📚 Additional Resources

- [Argo Events Official Documentation](https://argoproj.github.io/argo-events/)
- [Argo Events GitHub Examples](https://github.com/argoproj/argo-events/tree/master/examples)
- [Sensor Trigger Specification](https://argoproj.github.io/argo-events/concepts/trigger/)



---

**💡 Pro Tip:** When in doubt, grep these examples for the pattern you need instead of making assumptions about what Argo Events supports!
