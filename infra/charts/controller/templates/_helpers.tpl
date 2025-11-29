{{/*
Expand the name of the chart.
*/}}
{{- define "controller.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "controller.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "controller.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "controller.labels" -}}
helm.sh/chart: {{ include "controller.chart" . }}
{{ include "controller.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "controller.selectorLabels" -}}
app.kubernetes.io/name: {{ include "controller.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "controller.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "controller.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the RBAC role to use
*/}}
{{- define "controller.roleName" -}}
{{- if .Values.rbac.namespaced }}
{{- include "controller.fullname" . }}
{{- else }}
{{- include "controller.fullname" . }}
{{- end }}
{{- end }}

{{/*
Create a checksum for the tools catalog ConfigMap to force pod restart when it changes
*/}}
{{- define "controller.toolsCatalogChecksum" -}}
{{- $catalog := lookup "v1" "ConfigMap" .Release.Namespace "tools-tool-catalog" -}}
{{- if $catalog }}
{{- $catalog.data | toYaml | sha256sum }}
{{- else }}
{{- "not-found" }}
{{- end }}
{{- end }}

{{/*
Create a checksum for the tools catalog ConfigMap to force pod restart when it changes
*/}}
{{- define "orchestrator.toolsCatalogChecksum" -}}
{{- $catalog := lookup "v1" "ConfigMap" .Release.Namespace "tools-tool-catalog" -}}
{{- if $catalog }}
{{- $catalog.data | toYaml | sha256sum }}
{{- else }}
{{- "not-found" }}
{{- end }}
{{- end }}

{{/*
Define volumes for agent prompts ConfigMap
*/}}
{{- define "platform.agentVolumes" -}}
- name: agents-prompts
  configMap:
    name: {{ include "controller.fullname" . }}-agents
{{- end }}

{{/*
Define volume mounts for agent prompts
*/}}
{{- define "platform.agentVolumeMounts" -}}
- name: agents-prompts
  mountPath: /etc/agents
  readOnly: true
{{- end }}

{{/*
Define volume mounts for agent templates (all mounted to /agent-templates)
Note: Kubernetes projected volumes allow merging multiple ConfigMaps into one directory
*/}}
{{- define "platform.agentTemplateVolumeMounts" -}}
- name: agent-templates
  mountPath: /agent-templates
  readOnly: true
{{- end }}

{{/*
Define projected volume for agent templates (merges all ConfigMaps)
*/}}
{{- define "platform.agentTemplateProjectedVolume" -}}
- name: agent-templates
  projected:
    sources:
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-shared
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-claude-code
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-claude-docs
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-codex
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-cursor
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-factory
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-opencode
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-integration
        optional: true
    - configMap:
        name: {{ include "controller.fullname" . }}-agent-templates-watch
        optional: true
{{- end }}

{{/*
Define volume mounts for Blaze agent scripts
*/}}
{{- define "platform.blazeScriptsVolumeMounts" -}}
- name: blaze-scripts
  mountPath: /workspace/scripts/blaze
  readOnly: true
{{- end }}

{{/*
Define volume for Blaze agent scripts ConfigMap
*/}}
{{- define "platform.blazeScriptsVolume" -}}
- name: blaze-scripts
  configMap:
    name: {{ include "controller.fullname" . }}-agent-scripts-blaze
    defaultMode: 0755
    optional: true
{{- end }}
