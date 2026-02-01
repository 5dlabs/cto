{{/*
CTO Lite Helm Template Helpers
*/}}

{{/*
Expand the name of the chart.
*/}}
{{- define "cto-lite.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "cto-lite.fullname" -}}
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
{{- define "cto-lite.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "cto-lite.labels" -}}
helm.sh/chart: {{ include "cto-lite.chart" . }}
{{ include "cto-lite.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "cto-lite.selectorLabels" -}}
app.kubernetes.io/name: {{ include "cto-lite.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Controller labels
*/}}
{{- define "cto-lite.controller.labels" -}}
{{ include "cto-lite.labels" . }}
app.kubernetes.io/component: controller
{{- end }}

{{/*
Controller selector labels
*/}}
{{- define "cto-lite.controller.selectorLabels" -}}
{{ include "cto-lite.selectorLabels" . }}
app.kubernetes.io/component: controller
{{- end }}

{{/*
PM Lite labels
*/}}
{{- define "cto-lite.pm.labels" -}}
{{ include "cto-lite.labels" . }}
app.kubernetes.io/component: pm-lite
{{- end }}

{{/*
PM Lite selector labels
*/}}
{{- define "cto-lite.pm.selectorLabels" -}}
{{ include "cto-lite.selectorLabels" . }}
app.kubernetes.io/component: pm-lite
{{- end }}
