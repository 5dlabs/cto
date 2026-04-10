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

{{/*
Pod annotations — informational settings visible via kubectl describe pod
*/}}
{{- define "openclaw-agent.podAnnotations" -}}
openclaw.io/model: {{ .Values.agent.model | quote }}
{{- if kindIs "string" .Values.agent.heartbeat }}
openclaw.io/heartbeat: {{ .Values.agent.heartbeat | quote }}
{{- else if .Values.agent.heartbeat }}
openclaw.io/heartbeat: {{ .Values.agent.heartbeat.every | default "15m" | quote }}
{{- end }}
openclaw.io/sandbox: {{ .Values.agent.sandbox | quote }}
openclaw.io/tools-profile: {{ .Values.tools.profile | quote }}
{{- if .Values.datadog }}
{{- if .Values.datadog.enabled }}
ad.datadoghq.com/agent.logs: {{ printf "[{\"source\":\"openclaw-agent\",\"service\":\"cto-%s\",\"tags\":[\"agent:%s\",\"model:%s\"]}]" .Values.agent.id .Values.agent.id .Values.agent.model | quote }}
{{- end }}
{{- end }}
{{- with .Values.podAnnotations }}
{{ toYaml . }}
{{- end }}
{{- end }}
