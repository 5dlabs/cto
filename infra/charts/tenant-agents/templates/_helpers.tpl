{{/*
Expand the name of the chart.
*/}}
{{- define "tenant-agents.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "tenant-agents.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Values.tenant.id .Chart.Name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "tenant-agents.labels" -}}
helm.sh/chart: {{ include "tenant-agents.name" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
cto.5dlabs.ai/tenant: {{ .Values.tenant.id }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "tenant-agents.selectorLabels" -}}
app.kubernetes.io/name: {{ include "tenant-agents.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
cto.5dlabs.ai/tenant: {{ .Values.tenant.id }}
{{- end }}
