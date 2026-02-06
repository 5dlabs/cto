{{/*
Expand the name of the chart.
*/}}
{{- define "openclaw-agent.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name using agent ID.
*/}}
{{- define "openclaw-agent.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "openclaw-%s" .Values.agent.id | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "openclaw-agent.labels" -}}
helm.sh/chart: {{ include "openclaw-agent.name" . }}
{{ include "openclaw-agent.selectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: openclaw
openclaw.io/agent: {{ .Values.agent.id }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "openclaw-agent.selectorLabels" -}}
app.kubernetes.io/name: {{ include "openclaw-agent.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
