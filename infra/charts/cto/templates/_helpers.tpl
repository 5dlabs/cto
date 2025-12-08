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

{{- define "platform.agentTemplateProjectedVolume" -}}
- name: templates
  projected:
    sources:
      - configMap:
          name: controller-templates-shared
{{- end }}
