{{/*
Expand the name of the chart.
*/}}
{{- define "cto.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "cto.fullname" -}}
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
{{- define "cto.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "cto.labels" -}}
helm.sh/chart: {{ include "cto.chart" . }}
{{ include "cto.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: cto
{{- with .Values.global.labels }}
{{ toYaml . }}
{{- end }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "cto.selectorLabels" -}}
app.kubernetes.io/name: {{ include "cto.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Get the CTO namespace
*/}}
{{- define "cto.namespace" -}}
{{- .Values.global.namespace | default "cto" }}
{{- end }}

{{/*
Get the automation namespace (for sensors, webhooks)
*/}}
{{- define "cto.automationNamespace" -}}
{{- .Values.global.automationNamespace | default "automation" }}
{{- end }}

{{/*
Get the storage class
*/}}
{{- define "cto.storageClass" -}}
{{- .Values.global.storageClass | default "local-path" }}
{{- end }}

{{/*
Image pull secrets
*/}}
{{- define "cto.imagePullSecrets" -}}
{{- with .Values.global.imagePullSecrets }}
imagePullSecrets:
{{- toYaml . | nindent 2 }}
{{- end }}
{{- end }}

{{/*
PVC retention annotation
*/}}
{{- define "cto.pvcRetentionAnnotation" -}}
helm.sh/resource-policy: keep
{{- end }}

{{/*
Heal component labels
*/}}
{{- define "cto.heal.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/component: heal
{{- end }}

{{/*
Sensor component labels
*/}}
{{- define "cto.sensor.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/component: sensor
{{- end }}

{{/*
Webhook component labels
*/}}
{{- define "cto.webhook.labels" -}}
{{ include "cto.labels" . }}
app.kubernetes.io/component: webhook
{{- end }}


