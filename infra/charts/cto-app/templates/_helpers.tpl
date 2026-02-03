{{/*
Expand the name of the chart.
*/}}
{{- define "cto-app.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "cto-app.fullname" -}}
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
{{- define "cto-app.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "cto-app.labels" -}}
helm.sh/chart: {{ include "cto-app.chart" . }}
{{ include "cto-app.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "cto-app.selectorLabels" -}}
app.kubernetes.io/name: {{ include "cto-app.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "cto-app.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "cto-app.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Image tag
*/}}
{{- define "cto-app.imageTag" -}}
{{- if .Values.global.imageTag }}
{{- .Values.global.imageTag }}
{{- else if .Values.image.tag }}
{{- .Values.image.tag }}
{{- else if .Values.imageTag }}
{{- .Values.imageTag }}
{{- else }}
{{- "latest" }}
{{- end }}
{{- end }}
