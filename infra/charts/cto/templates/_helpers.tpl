{{/*
CTO Platform - Shared Helm Helpers
*/}}

{{/*
Expand the name of the chart.
*/}}
{{- define "cto.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "cto.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- printf "%s" $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "cto.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "cto.labels" -}}
helm.sh/chart: {{ include "cto.chart" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: cto-platform
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}

{{/*
Namespace - uses global.namespace or Release.Namespace
*/}}
{{- define "cto.namespace" -}}
{{- ((.Values.global).namespace) | default .Release.Namespace }}
{{- end }}

{{/*
Image pull secrets
*/}}
{{- define "cto.imagePullSecrets" -}}
{{- with ((.Values.global).imagePullSecrets) }}
imagePullSecrets:
  {{- toYaml . | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Image reference - returns full image:tag based on devRegistry toggle
Usage: {{ include "cto.imageRef" (dict "component" "controller" "image" .Values.controller.image "global" .Values.global) }}

When global.devRegistry.enabled is true:
  - Uses local registry URL from global.devRegistry.url
  - Uses tag from global.devRegistry.componentTags[component] or global.devRegistry.tag
When global.devRegistry.enabled is false:
  - Uses the component's configured image.repository and image.tag
*/}}
{{- define "cto.imageRef" -}}
{{- $component := .component -}}
{{- $image := .image -}}
{{- $global := .global -}}
{{- $devRegistry := ($global).devRegistry | default dict -}}
{{- $enabled := ($devRegistry).enabled | default false -}}
{{- if $enabled -}}
{{- $url := ($devRegistry).url | default "localhost:30500" -}}
{{- $componentTags := ($devRegistry).componentTags | default dict -}}
{{- $tag := index $componentTags $component | default ($devRegistry).tag | default "dev-local" -}}
{{- printf "%s/%s:%s" $url $component $tag -}}
{{- else -}}
{{- printf "%s:%s" $image.repository $image.tag -}}
{{- end -}}
{{- end }}

{{/*
Image pull policy - returns pullPolicy based on devRegistry toggle
Usage: {{ include "cto.imagePullPolicy" (dict "image" .Values.controller.image "global" .Values.global) }}
*/}}
{{- define "cto.imagePullPolicy" -}}
{{- $image := .image -}}
{{- $global := .global -}}
{{- $devRegistry := ($global).devRegistry | default dict -}}
{{- $enabled := ($devRegistry).enabled | default false -}}
{{- if $enabled -}}
{{- ($devRegistry).pullPolicy | default "Always" -}}
{{- else -}}
{{- $image.pullPolicy | default "IfNotPresent" -}}
{{- end -}}
{{- end }}

{{/* ============================================================ */}}
{{/* Component-specific helpers */}}
{{/* ============================================================ */}}

{{/* Controller */}}
{{- define "cto.controller.fullname" -}}
{{- printf "%s-controller" (include "cto.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "cto.controller.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/name: controller
app.kubernetes.io/component: orchestration
{{- end }}

{{- define "cto.controller.selectorLabels" -}}
app.kubernetes.io/name: controller
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/* PM */}}
{{- define "cto.pm.fullname" -}}
{{- printf "%s-pm" (include "cto.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "cto.pm.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/name: pm
app.kubernetes.io/component: project-management
{{- end }}

{{- define "cto.pm.selectorLabels" -}}
app.kubernetes.io/name: pm
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/* Tools */}}
{{- define "cto.tools.fullname" -}}
{{- printf "%s-tools" (include "cto.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "cto.tools.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/name: tools
app.kubernetes.io/component: mcp-proxy
{{- end }}

{{- define "cto.tools.selectorLabels" -}}
app.kubernetes.io/name: tools
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/* Healer */}}
{{- define "cto.healer.fullname" -}}
{{- printf "%s-healer" (include "cto.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "cto.healer.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/name: healer
app.kubernetes.io/component: self-healing
{{- end }}

{{- define "cto.healer.selectorLabels" -}}
app.kubernetes.io/name: healer
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/* OpenMemory */}}
{{- define "cto.openmemory.fullname" -}}
{{- printf "%s-openmemory" (include "cto.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "cto.openmemory.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/name: openmemory
app.kubernetes.io/component: memory
{{- end }}

{{- define "cto.openmemory.selectorLabels" -}}
app.kubernetes.io/name: openmemory
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/* ============================================================ */}}
{{/* Platform/Agent helpers for workflow templates */}}
{{/* ============================================================ */}}

{{- define "platform.agentVolumeMounts" -}}
- name: workspace
  mountPath: /workspace
- name: tmp
  mountPath: /tmp
{{- end }}

{{- define "platform.agentTemplateVolumeMounts" -}}
- name: templates
  mountPath: /templates
  readOnly: true
{{- end }}

{{- define "platform.agentVolumes" -}}
- name: workspace
  emptyDir: {}
- name: tmp
  emptyDir: {}
{{- end }}
