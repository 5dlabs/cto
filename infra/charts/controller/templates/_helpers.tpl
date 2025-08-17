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
Create a checksum for the toolman catalog ConfigMap to force pod restart when it changes
*/}}
{{- define "controller.toolmanCatalogChecksum" -}}
{{- $catalog := lookup "v1" "ConfigMap" .Release.Namespace "toolman-tool-catalog" -}}
{{- if $catalog }}
{{- $catalog.data | toYaml | sha256sum }}
{{- else }}
{{- "not-found" }}
{{- end }}
{{- end }}

{{/*
Create a checksum for the toolman catalog ConfigMap to force pod restart when it changes
*/}}
{{- define "orchestrator.toolmanCatalogChecksum" -}}
{{- $catalog := lookup "v1" "ConfigMap" .Release.Namespace "toolman-tool-catalog" -}}
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