# CTO Tools Catalog

> Auto-generated from production toolserver at `https://tools.fra.5dlabs.ai`
> Generated: 2026-01-30T14:37 PST

## Summary

| Server | Tool Count |
|--------|------------|
| `grafana` | 56 |
| `kubernetes` | 22 |
| `playwright` | 22 |
| `octocode` | 13 |
| `terraform` | 9 |
| `openmemory` | 6 |
| `prometheus` | 6 |
| `better` | 4 |
| `exa` | 3 |
| `solana` | 3 |
| `ai` | 2 |
| `graphql` | 2 |
| `pg` | 2 |
| `tools` | 2 |
| **Total** | **152** |

---

## Grafana (56 tools)

### `grafana_add_activity_to_incident`
Add a note (userNote activity) to an existing incident's timeline using its ID. The note body can include URLs which will be attached as context. Use this to add context to an incident.

**Parameters:**
- `body` (string)  — The body of the activity. URLs will be parsed and attached as context
- `eventTime` (string)  — The time that the activity occurred. If not provided, the current time will be u...
- `incidentId` (string)  — The ID of the incident to add the activity to

---

### `grafana_create_alert_rule`
Creates a new Grafana alert rule with the specified configuration. Requires title, rule group, folder UID, condition, query data, no data state, execution error state, and duration settings.

**Parameters:**
- `annotations` (object)  — Optional annotations
- `condition` (string) ✅ — The query condition identifier (e.g. 'A', 'B')
- `data` (any) ✅ — Array of query data objects
- `execErrState` (string) ✅ — State on execution error (NoData, Alerting, OK)
- `folderUID` (string) ✅ — The folder UID where the rule will be created
- `for` (string) ✅ — Duration before alert fires (e.g. '5m')
- `labels` (object)  — Optional labels
- `noDataState` (string) ✅ — State when no data (NoData, Alerting, OK)
- `orgID` (integer) ✅ — The organization ID
- `ruleGroup` (string) ✅ — The rule group name
- `title` (string) ✅ — The title of the alert rule
- `uid` (string)  — Optional UID for the alert rule

---

### `grafana_create_annotation`
Create a new annotation on a dashboard or panel.

**Parameters:**
- `dashboardId` (integer)  — Deprecated. Use dashboardUID
- `dashboardUID` (string)  — Preferred dashboard UID
- `data` (object)  — Optional JSON payload
- `panelId` (integer)  — Panel ID
- `tags` (array)  — Optional list of tags
- `text` (string)  — Annotation text required
- `time` (integer)  — Start time epoch ms
- `timeEnd` (integer)  — End time epoch ms

---

### `grafana_create_folder`
Create a Grafana folder. Provide a title and optional UID. Returns the created folder.

**Parameters:**
- `parentUid` (string)  — Optional parent folder UID. If set, the folder will be created under this parent...
- `title` (string) ✅ — The title of the folder.
- `uid` (string)  — Optional folder UID. If omitted, Grafana will generate one.

---

### `grafana_create_graphite_annotation`
Create an annotation using Graphite annotation format.

**Parameters:**
- `data` (string)  — Optional payload
- `tags` (array)  — Optional list of tags
- `what` (string)  — Annotation text
- `when` (integer)  — Epoch ms timestamp

---

### `grafana_create_incident`
Create a new Grafana incident. Requires title, severity, and room prefix. Allows setting status and labels. This tool should be used judiciously and sparingly, and only after confirmation from the use...

**Parameters:**
- `attachCaption` (string)  — The caption of the attachment
- `attachUrl` (string)  — The URL of the attachment
- `isDrill` (boolean)  — Whether the incident is a drill incident
- `labels` (array)  — The labels to add to the incident
- `roomPrefix` (string)  — The prefix of the room to create the incident in
- `severity` (string)  — The severity of the incident
- `status` (string)  — The status of the incident
- `title` (string)  — The title of the incident

---

### `grafana_delete_alert_rule`
Deletes a Grafana alert rule by its UID. This action cannot be undone.

**Parameters:**
- `uid` (string) ✅ — The UID of the alert rule to delete

---

### `grafana_fetch_pyroscope_profile`
...

**Parameters:**
- `data_source_uid` (string) ✅ — The UID of the datasource to query
- `end_rfc_3339` (string)  — Optionally, the end time of the query in RFC3339 format (defaults to now)
- `matchers` (string)  — Optionally, Prometheus style matchers used to filter the result set (defaults to...
- `max_node_depth` (integer)  — Optionally, the maximum depth of nodes in the resulting profile. Less depth resu...
- `profile_type` (string) ✅ — Type profile type, use the list_pyroscope_profile_types tool to fetch available ...
- `start_rfc_3339` (string)  — Optionally, the start time of the query in RFC3339 format (defaults to 1 hour ag...

---

### `grafana_find_error_pattern_logs`
Searches Loki logs for elevated error patterns compared to the last day's average, waits for the analysis to complete, and returns the results including any patterns found.

**Parameters:**
- `end` (string)  — End time for the investigation. Defaults to now if not specified.
- `labels` (object) ✅ — Labels to scope the analysis
- `name` (string) ✅ — The name of the investigation
- `start` (string)  — Start time for the investigation. Defaults to 30 minutes ago if not specified.

---

### `grafana_find_slow_requests`
Searches relevant Tempo datasources for slow requests, waits for the analysis to complete, and returns the results.

**Parameters:**
- `end` (string)  — End time for the investigation. Defaults to now if not specified.
- `labels` (object) ✅ — Labels to scope the analysis
- `name` (string) ✅ — The name of the investigation
- `start` (string)  — Start time for the investigation. Defaults to 30 minutes ago if not specified.

---

### `grafana_generate_deeplink`
Generate deeplink URLs for Grafana resources. Supports dashboards (requires dashboardUid), panels (requires dashboardUid and panelId), and Explore queries (requires datasourceUid). Optionally accepts ...

**Parameters:**
- `dashboardUid` (string)  — Dashboard UID (required for dashboard and panel types)
- `datasourceUid` (string)  — Datasource UID (required for explore type)
- `panelId` (integer)  — Panel ID (required for panel type)
- `queryParams` (object)  — Additional query parameters
- `resourceType` (string) ✅ — Type of resource: dashboard, panel, or explore
- `timeRange` (object)  — Time range for the link

---

### `grafana_get_alert_group`
Get a specific alert group from Grafana OnCall by its ID. Returns the full alert group details.

**Parameters:**
- `alertGroupId` (string) ✅ — The ID of the alert group to retrieve

---

### `grafana_get_alert_rule_by_uid`
Retrieves the full configuration and detailed status of a specific Grafana alert rule identified by its unique ID (UID). The response includes fields like title, condition, query data, folder UID, rul...

**Parameters:**
- `uid` (string) ✅ — The uid of the alert rule

---

### `grafana_get_annotation_tags`
Returns annotation tags with optional filtering by tag name. Only the provided filters are applied.

**Parameters:**
- `limit` (string)  — Max results, default 100
- `tag` (string)  — Optional filter by tag name

---

### `grafana_get_annotations`
Fetch Grafana annotations using filters such as dashboard UID, time range and tags.

**Parameters:**
- `AlertID` (integer)  — Deprecated. Use AlertUID
- `AlertUID` (string)  — Filter by alert UID
- `DashboardID` (integer)  — Deprecated. Use DashboardUID
- `DashboardUID` (string)  — Filter by dashboard UID
- `From` (integer)  — Epoch ms start time
- `Limit` (integer)  — Max results default 100
- `MatchAny` (boolean)  — true OR tag match false AND
- `PanelID` (integer)  — Filter by panel ID
- `Tags` (array)  — Multiple tags allowed tags=tag1&tags=tag2
- `To` (integer)  — Epoch ms end time
- `Type` (string)  — annotation or alert
- `UserID` (integer)  — Filter by creator user ID

---

### `grafana_get_assertions`
Get assertion summary for a given entity with its type, name, env, site, namespace, and a time range

**Parameters:**
- `endTime` (string) ✅ — The end time in RFC3339 format
- `entityName` (string)  — The name of the entity to list
- `entityType` (string)  — The type of the entity to list (e.g. Service, Node, Pod, etc.)
- `env` (string)  — The env of the entity to list
- `namespace` (string)  — The namespace of the entity to list
- `site` (string)  — The site of the entity to list
- `startTime` (string) ✅ — The start time in RFC3339 format

---

### `grafana_get_current_oncall_users`
Get the list of users currently on-call for a specific Grafana OnCall schedule ID. Returns the schedule ID, name, and a list of detailed user objects for those currently on call.

**Parameters:**
- `scheduleId` (string) ✅ — The ID of the schedule to get current on-call users for

---

### `grafana_get_dashboard_by_uid`
Retrieves the complete dashboard, including panels, variables, and settings, for a specific dashboard identified by its UID. WARNING: Large dashboards can consume significant context window space. Con...

**Parameters:**
- `uid` (string) ✅ — The UID of the dashboard

---

### `grafana_get_dashboard_panel_queries`
Use this tool to retrieve panel queries and information from a Grafana dashboard. When asked about panel queries, queries in a dashboard, or what queries a dashboard contains, call this tool with the ...

**Parameters:**
- `uid` (string) ✅ — The UID of the dashboard

---

### `grafana_get_dashboard_property`
Get specific parts of a dashboard using JSONPath expressions to minimize context window usage. Common paths: '$.title' (title)\, '$.panels[*].title' (all panel titles)\, '$.panels[0]' (first panel)\, ...

**Parameters:**
- `jsonPath` (string) ✅ — JSONPath expression to extract specific data (e.g., '$.panels[0].title' for firs...
- `uid` (string) ✅ — The UID of the dashboard

---

### `grafana_get_dashboard_summary`
Get a compact summary of a dashboard including title\, panel count\, panel types\, variables\, and other metadata without the full JSON. Use this for dashboard overview and planning modifications with...

**Parameters:**
- `uid` (string) ✅ — The UID of the dashboard

---

### `grafana_get_datasource_by_name`
Retrieves detailed information about a specific datasource using its name. Returns the full datasource model, including UID, type, URL, access settings, JSON data, and secure JSON field status.

**Parameters:**
- `name` (string) ✅ — The name of the datasource

---

### `grafana_get_datasource_by_uid`
Retrieves detailed information about a specific datasource using its UID. Returns the full datasource model, including name, type, URL, access settings, JSON data, and secure JSON field status.

**Parameters:**
- `uid` (string) ✅ — The uid of the datasource

---

### `grafana_get_incident`
Get a single incident by ID. Returns the full incident details including title, status, severity, labels, timestamps, and other metadata.

**Parameters:**
- `id` (string)  — The ID of the incident to retrieve

---

### `grafana_get_oncall_shift`
Get detailed information for a specific Grafana OnCall shift using its ID. A shift represents a designated time period within a schedule when users are actively on-call. Returns the full shift details...

**Parameters:**
- `shiftId` (string) ✅ — The ID of the shift to get details for

---

### `grafana_get_sift_analysis`
Retrieves a specific analysis from an investigation by its UUID. The investigation ID and analysis ID should be provided as strings in UUID format.

**Parameters:**
- `analysisId` (string) ✅ — The UUID of the specific analysis to retrieve
- `investigationId` (string) ✅ — The UUID of the investigation as a string (e.g. '02adab7c-bf5b-45f2-9459-d71a2c2...

---

### `grafana_get_sift_investigation`
Retrieves an existing Sift investigation by its UUID. The ID should be provided as a string in UUID format (e.g. '02adab7c-bf5b-45f2-9459-d71a2c29e11b').

**Parameters:**
- `id` (string) ✅ — The UUID of the investigation as a string (e.g. '02adab7c-bf5b-45f2-9459-d71a2c2...

---

### `grafana_list_alert_groups`
List alert groups from Grafana OnCall with filtering options. Supports filtering by alert group ID, route ID, integration ID, state (new, acknowledged, resolved, silenced), team ID, time range, labels...

**Parameters:**
- `id` (string)  — Filter by specific alert group ID
- `integrationId` (string)  — Filter by integration ID
- `labels` (array)  — Filter by labels in format key:value (e.g., ['env:prod', 'severity:high'])
- `name` (string)  — Filter by alert group name
- `page` (integer)  — The page number to return
- `routeId` (string)  — Filter by route ID
- `startedAt` (string)  — Filter by time range in format '{start}_{end}' ISO 8601 timestamp range (UTC ass...
- `state` (string)  — Filter by alert group state (one of: new, acknowledged, resolved, silenced)
- `teamId` (string)  — Filter by team ID

---

### `grafana_list_alert_rules`
Lists Grafana alert rules, returning a summary including UID, title, current state (e.g., 'pending', 'firing', 'inactive'), and labels. Optionally query datasource-managed rules from Prometheus or Lok...

**Parameters:**
- `datasourceUid` (string)  — Optional: UID of a Prometheus or Loki datasource to query for datasource-managed...
- `label_selectors` (array)  — Optionally, a list of matchers to filter alert rules by labels
- `limit` (integer)  — The maximum number of results to return
- `page` (integer)  — The page number to return

---

### `grafana_list_contact_points`
Lists Grafana notification contact points, returning a summary including UID, name, and type for each. Optionally query Alertmanager receivers by providing datasourceUid. Supports filtering by name - ...

**Parameters:**
- `datasourceUid` (string)  — Optional: UID of an Alertmanager-compatible datasource to query for receivers. I...
- `limit` (integer)  — The maximum number of results to return. Default is 100.
- `name` (string)  — Filter contact points by name

---

### `grafana_list_datasources`
List available Grafana datasources. Optionally filter by datasource type (e.g., 'prometheus', 'loki'). Returns a summary list including ID, UID, name, type, and default status.

**Parameters:**
- `type` (string)  — The type of datasources to search for. For example, 'prometheus', 'loki', 'tempo...

---

### `grafana_list_incidents`
List Grafana incidents. Allows filtering by status ('active', 'resolved') and optionally including drill incidents. Returns a preview list with basic details.

**Parameters:**
- `drill` (boolean)  — Whether to include drill incidents
- `limit` (integer)  — The maximum number of incidents to return
- `status` (string)  — The status of the incidents to include. Valid values: 'active', 'resolved'

---

### `grafana_list_loki_label_names`
Lists all available label names (keys) found in logs within a specified Loki datasource and time range. Returns a list of unique label strings (e.g., `["app", "env", "pod"]`). If the time range is not...

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `endRfc3339` (string)  — Optionally, the end time of the query in RFC3339 format (defaults to now)
- `startRfc3339` (string)  — Optionally, the start time of the query in RFC3339 format (defaults to 1 hour ag...

---

### `grafana_list_loki_label_values`
Retrieves all unique values associated with a specific `labelName` within a Loki datasource and time range. Returns a list of string values (e.g., for `labelName="env"`, might return `["prod", "stagin...

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `endRfc3339` (string)  — Optionally, the end time of the query in RFC3339 format (defaults to now)
- `labelName` (string) ✅ — The name of the label to retrieve values for (e.g. 'app', 'env', 'pod')
- `startRfc3339` (string)  — Optionally, the start time of the query in RFC3339 format (defaults to 1 hour ag...

---

### `grafana_list_oncall_schedules`
List Grafana OnCall schedules, optionally filtering by team ID. If a specific schedule ID is provided, retrieves details for only that schedule. Returns a list of schedule summaries including ID, name...

**Parameters:**
- `page` (integer)  — The page number to return (1-based)
- `scheduleId` (string)  — The ID of the schedule to get details for. If provided, returns only that schedu...
- `teamId` (string)  — The ID of the team to list schedules for

---

### `grafana_list_oncall_teams`
List teams configured in Grafana OnCall. Returns a list of team objects with their details. Supports pagination.

**Parameters:**
- `page` (integer)  — The page number to return

---

### `grafana_list_oncall_users`
List users from Grafana OnCall. Can retrieve all users, a specific user by ID, or filter by username. Returns a list of user objects with their details. Supports pagination.

**Parameters:**
- `page` (integer)  — The page number to return
- `userId` (string)  — The ID of the user to get details for. If provided, returns only that user's det...
- `username` (string)  — The username to filter users by. If provided, returns only the user matching thi...

---

### `grafana_list_prometheus_label_names`
List label names in a Prometheus datasource. Allows filtering by series selectors and time range.

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `endRfc3339` (string)  — Optionally, the end time of the time range to filter the results by
- `limit` (integer)  — Optionally, the maximum number of results to return
- `matches` (array)  — Optionally, a list of label matchers to filter the results by
- `startRfc3339` (string)  — Optionally, the start time of the time range to filter the results by

---

### `grafana_list_prometheus_label_values`
Get the values for a specific label name in Prometheus. Allows filtering by series selectors and time range.

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `endRfc3339` (string)  — Optionally, the end time of the query
- `labelName` (string) ✅ — The name of the label to query
- `limit` (integer)  — Optionally, the maximum number of results to return
- `matches` (array)  — Optionally, a list of selectors to filter the results by
- `startRfc3339` (string)  — Optionally, the start time of the query

---

### `grafana_list_prometheus_metric_metadata`
List Prometheus metric metadata. Returns metadata about metrics currently scraped from targets. Note: This endpoint is experimental.

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `limit` (integer)  — The maximum number of metrics to return
- `limitPerMetric` (integer)  — The maximum number of metrics to return per metric
- `metric` (string)  — The metric to query

---

### `grafana_list_prometheus_metric_names`
List metric names in a Prometheus datasource. Retrieves all metric names and then filters them locally using the provided regex. Supports pagination.

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `limit` (integer)  — The maximum number of results to return
- `page` (integer)  — The page number to return
- `regex` (string)  — The regex to match against the metric names

---

### `grafana_list_pyroscope_label_names`
...

**Parameters:**
- `data_source_uid` (string) ✅ — The UID of the datasource to query
- `end_rfc_3339` (string)  — Optionally, the end time of the query in RFC3339 format (defaults to now)
- `matchers` (string)  — 
- `start_rfc_3339` (string)  — Optionally, the start time of the query in RFC3339 format (defaults to 1 hour ag...

---

### `grafana_list_pyroscope_label_values`
...

**Parameters:**
- `data_source_uid` (string) ✅ — The UID of the datasource to query
- `end_rfc_3339` (string)  — Optionally, the end time of the query in RFC3339 format (defaults to now)
- `matchers` (string)  — Optionally, Prometheus style matchers used to filter the result set (defaults to...
- `name` (string) ✅ — A label name
- `start_rfc_3339` (string)  — Optionally, the start time of the query in RFC3339 format (defaults to 1 hour ag...

---

### `grafana_list_pyroscope_profile_types`
...

**Parameters:**
- `data_source_uid` (string) ✅ — The UID of the datasource to query
- `end_rfc_3339` (string)  — Optionally, the end time of the query in RFC3339 format (defaults to now)
- `start_rfc_3339` (string)  — Optionally, the start time of the query in RFC3339 format (defaults to 1 hour ag...

---

### `grafana_list_sift_investigations`
Retrieves a list of Sift investigations with an optional limit. If no limit is specified, defaults to 10 investigations.

**Parameters:**
- `limit` (integer)  — Maximum number of investigations to return

---

### `grafana_list_teams`
Search for Grafana teams by a query string. Returns a list of matching teams with details like name, ID, and URL.

**Parameters:**
- `query` (string)  — The query to search for teams. Can be left empty to fetch all teams

---

### `grafana_list_users_by_org`
List users by organization. Returns a list of users with details like userid, email, role etc.

**Parameters:** *(none)*

---

### `grafana_patch_annotation`
Updates only the provided properties of an annotation. Fields omitted are not modified. Use update_annotation for full replacement.

**Parameters:**
- `data` (object)  — Optional metadata
- `id` (integer)  — Annotation ID
- `tags` (array)  — Optional replace tags
- `text` (string)  — Optional new text
- `time` (integer)  — Optional new start epoch ms
- `timeEnd` (integer)  — Optional new end epoch ms

---

### `grafana_query_loki_logs`
Executes a LogQL query against a Loki datasource to retrieve log entries or metric values. Returns a list of results, each containing a timestamp, labels, and either a log line (`line`) or a numeric m...

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `direction` (string)  — Optionally, the direction of the query: 'forward' (oldest first) or 'backward' (...
- `endRfc3339` (string)  — Optionally, the end time of the query in RFC3339 format
- `limit` (integer)  — Optionally, the maximum number of log lines to return (max: 100)
- `logql` (string) ✅ — The LogQL query to execute against Loki. This can be a simple label matcher or a...
- `startRfc3339` (string)  — Optionally, the start time of the query in RFC3339 format

---

### `grafana_query_loki_stats`
Retrieves statistics about log streams matching a given LogQL *selector* within a Loki datasource and time range. Returns an object containing the count of streams, chunks, entries, and total bytes (e...

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `endRfc3339` (string)  — Optionally, the end time of the query in RFC3339 format
- `logql` (string) ✅ — The LogQL matcher expression to execute. This parameter only accepts label match...
- `startRfc3339` (string)  — Optionally, the start time of the query in RFC3339 format

---

### `grafana_query_prometheus`
Query Prometheus using a PromQL expression. Supports both instant queries (at a single point in time) and range queries (over a time range). Time can be specified either in RFC3339 format or as relati...

**Parameters:**
- `datasourceUid` (string) ✅ — The UID of the datasource to query
- `endTime` (string)  — The end time. Required if queryType is 'range', ignored if queryType is 'instant...
- `expr` (string) ✅ — The PromQL expression to query
- `queryType` (string)  — The type of query to use. Either 'range' or 'instant'
- `startTime` (string) ✅ — The start time. Supported formats are RFC3339 or relative to now (e.g. 'now', 'n...
- `stepSeconds` (integer)  — The time series step size in seconds. Required if queryType is 'range', ignored ...

---

### `grafana_search_dashboards`
Search for Grafana dashboards by a query string. Returns a list of matching dashboards with details like title, UID, folder, tags, and URL.

**Parameters:**
- `query` (string)  — The query to search for

---

### `grafana_search_folders`
Search for Grafana folders by a query string. Returns matching folders with details like title, UID, and URL.

**Parameters:**
- `query` (string)  — The query to search for

---

### `grafana_update_alert_rule`
Updates an existing Grafana alert rule identified by its UID. Requires all the same parameters as creating a new rule.

**Parameters:**
- `annotations` (object)  — Optional annotations
- `condition` (string) ✅ — The query condition identifier (e.g. 'A', 'B')
- `data` (any) ✅ — Array of query data objects
- `execErrState` (string) ✅ — State on execution error (NoData, Alerting, OK)
- `folderUID` (string) ✅ — The folder UID where the rule will be created
- `for` (string) ✅ — Duration before alert fires (e.g. '5m')
- `labels` (object)  — Optional labels
- `noDataState` (string) ✅ — State when no data (NoData, Alerting, OK)
- `orgID` (integer) ✅ — The organization ID
- `ruleGroup` (string) ✅ — The rule group name
- `title` (string) ✅ — The title of the alert rule
- `uid` (string) ✅ — The UID of the alert rule to update

---

### `grafana_update_annotation`
Updates all properties of an annotation that matches the specified ID. Sends a full update (PUT). For partial updates, use patch_annotation instead.

**Parameters:**
- `data` (object)  — Optional JSON payload
- `id` (integer)  — Annotation ID to update
- `tags` (array)  — Tags to replace existing tags
- `text` (string)  — Annotation text
- `time` (integer)  — Start time epoch ms
- `timeEnd` (integer)  — End time epoch ms

---

### `grafana_update_dashboard`
Create or update a dashboard using either full JSON or efficient patch operations. For new dashboards\, provide the 'dashboard' field. For updating existing dashboards\, use 'uid' + 'operations' for b...

**Parameters:**
- `dashboard` (object)  — The full dashboard JSON. Use for creating new dashboards or complete updates. La...
- `folderUid` (string)  — The UID of the dashboard's folder
- `message` (string)  — Set a commit message for the version history
- `operations` (array)  — Array of patch operations for targeted updates. More efficient than full dashboa...
- `overwrite` (boolean)  — Overwrite the dashboard if it exists. Otherwise create one
- `uid` (string)  — UID of existing dashboard to update. Required when using patch operations.
- `userId` (integer)  — ID of the user making the change

---

## Kubernetes (22 tools)

### `kubernetes_mcp_configuration_view`
Get the current Kubernetes configuration content as a kubeconfig YAML

**Parameters:**
- `minified` (boolean)  — Return a minified version of the configuration. If set to true, keeps only the c...

---

### `kubernetes_mcp_events_list`
List all the Kubernetes events in the current cluster from all namespaces

**Parameters:**
- `namespace` (string)  — Optional Namespace to retrieve the events from. If not provided, will list event...

---

### `kubernetes_mcp_helm_install`
Install a Helm chart in the current or provided namespace

**Parameters:**
- `chart` (string) ✅ — Chart reference to install (for example: stable/grafana, oci://ghcr.io/nginxinc/...
- `name` (string)  — Name of the Helm release (Optional, random name if not provided)
- `namespace` (string)  — Namespace to install the Helm chart in (Optional, current namespace if not provi...
- `values` (object)  — Values to pass to the Helm chart (Optional)

---

### `kubernetes_mcp_helm_list`
List all the Helm releases in the current or provided namespace (or in all namespaces if specified)

**Parameters:**
- `all_namespaces` (boolean)  — If true, lists all Helm releases in all namespaces ignoring the namespace argume...
- `namespace` (string)  — Namespace to list Helm releases from (Optional, all namespaces if not provided)

---

### `kubernetes_mcp_helm_uninstall`
Uninstall a Helm release in the current or provided namespace

**Parameters:**
- `name` (string) ✅ — Name of the Helm release to uninstall
- `namespace` (string)  — Namespace to uninstall the Helm release from (Optional, current namespace if not...

---

### `kubernetes_mcp_namespaces_list`
List all the Kubernetes namespaces in the current cluster

**Parameters:** *(none)*

---

### `kubernetes_mcp_nodes_log`
Get logs from a Kubernetes node (kubelet, kube-proxy, or other system logs). This accesses node logs through the Kubernetes API proxy to the kubelet

**Parameters:**
- `name` (string) ✅ — Name of the node to get logs from
- `query` (string) ✅ — query specifies services(s) or files from which to return logs (required). Examp...
- `tailLines` (integer)  — Number of lines to retrieve from the end of the logs (Optional, 0 means all logs...

---

### `kubernetes_mcp_nodes_stats_summary`
Get detailed resource usage statistics from a Kubernetes node via the kubelet's Summary API. Provides comprehensive metrics including CPU, memory, filesystem, and network usage at the node, pod, and c...

**Parameters:**
- `name` (string) ✅ — Name of the node to get stats from

---

### `kubernetes_mcp_nodes_top`
List the resource consumption (CPU and memory) as recorded by the Kubernetes Metrics Server for the specified Kubernetes Nodes or all nodes in the cluster

**Parameters:**
- `label_selector` (string)  — Kubernetes label selector (e.g. 'node-role.kubernetes.io/worker=') to filter nod...
- `name` (string)  — Name of the Node to get the resource consumption from (Optional, all Nodes if no...

---

### `kubernetes_mcp_pods_delete`
Delete a Kubernetes Pod in the current or provided namespace with the provided name

**Parameters:**
- `name` (string) ✅ — Name of the Pod to delete
- `namespace` (string)  — Namespace to delete the Pod from

---

### `kubernetes_mcp_pods_exec`
Execute a command in a Kubernetes Pod in the current or provided namespace with the provided name and command

**Parameters:**
- `command` (array) ✅ — Command to execute in the Pod container. The first item is the command to be run...
- `container` (string)  — Name of the Pod container where the command will be executed (Optional)
- `name` (string) ✅ — Name of the Pod where the command will be executed
- `namespace` (string)  — Namespace of the Pod where the command will be executed

---

### `kubernetes_mcp_pods_get`
Get a Kubernetes Pod in the current or provided namespace with the provided name

**Parameters:**
- `name` (string) ✅ — Name of the Pod
- `namespace` (string)  — Namespace to get the Pod from

---

### `kubernetes_mcp_pods_list`
List all the Kubernetes pods in the current cluster from all namespaces

**Parameters:**
- `fieldSelector` (string)  — Optional Kubernetes field selector to filter pods by field values (e.g. 'status....
- `labelSelector` (string)  — Optional Kubernetes label selector (e.g. 'app=myapp,env=prod' or 'app in (myapp,...

---

### `kubernetes_mcp_pods_list_in_namespace`
List all the Kubernetes pods in the specified namespace in the current cluster

**Parameters:**
- `fieldSelector` (string)  — Optional Kubernetes field selector to filter pods by field values (e.g. 'status....
- `labelSelector` (string)  — Optional Kubernetes label selector (e.g. 'app=myapp,env=prod' or 'app in (myapp,...
- `namespace` (string) ✅ — Namespace to list pods from

---

### `kubernetes_mcp_pods_log`
Get the logs of a Kubernetes Pod in the current or provided namespace with the provided name

**Parameters:**
- `container` (string)  — Name of the Pod container to get the logs from (Optional)
- `name` (string) ✅ — Name of the Pod to get the logs from
- `namespace` (string)  — Namespace to get the Pod logs from
- `previous` (boolean)  — Return previous terminated container logs (Optional)
- `tail` (integer)  — Number of lines to retrieve from the end of the logs (Optional, default: 100)

---

### `kubernetes_mcp_pods_run`
Run a Kubernetes Pod in the current or provided namespace with the provided container image and optional name

**Parameters:**
- `image` (string) ✅ — Container Image to run in the Pod
- `name` (string)  — Name of the Pod (Optional, random name if not provided)
- `namespace` (string)  — Namespace to run the Pod in
- `port` (number)  — TCP/IP port to expose from the Pod container (Optional, no port exposed if not p...

---

### `kubernetes_mcp_pods_top`
List the resource consumption (CPU and memory) as recorded by the Kubernetes Metrics Server for the specified Kubernetes Pods in the all namespaces, the provided namespace, or the current namespace

**Parameters:**
- `all_namespaces` (boolean)  — If true, list the resource consumption for all Pods in all namespaces. If false,...
- `label_selector` (string)  — Kubernetes label selector (e.g. 'app=myapp,env=prod' or 'app in (myapp,yourapp)'...
- `name` (string)  — Name of the Pod to get the resource consumption from (Optional, all Pods in the ...
- `namespace` (string)  — Namespace to get the Pods resource consumption from (Optional, current namespace...

---

### `kubernetes_mcp_resources_create_or_update`
Create or update a Kubernetes resource in the current cluster by providing a YAML or JSON representation of the resource...

**Parameters:**
- `resource` (string) ✅ — A JSON or YAML containing a representation of the Kubernetes resource. Should in...

---

### `kubernetes_mcp_resources_delete`
Delete a Kubernetes resource in the current cluster by providing its apiVersion, kind, optionally the namespace, and its name...

**Parameters:**
- `apiVersion` (string) ✅ — apiVersion of the resource (examples of valid apiVersion are: v1, apps/v1, netwo...
- `kind` (string) ✅ — kind of the resource (examples of valid kind are: Pod, Service, Deployment, Ingr...
- `name` (string) ✅ — Name of the resource
- `namespace` (string)  — Optional Namespace to delete the namespaced resource from (ignored in case of cl...

---

### `kubernetes_mcp_resources_get`
Get a Kubernetes resource in the current cluster by providing its apiVersion, kind, optionally the namespace, and its name...

**Parameters:**
- `apiVersion` (string) ✅ — apiVersion of the resource (examples of valid apiVersion are: v1, apps/v1, netwo...
- `kind` (string) ✅ — kind of the resource (examples of valid kind are: Pod, Service, Deployment, Ingr...
- `name` (string) ✅ — Name of the resource
- `namespace` (string)  — Optional Namespace to retrieve the namespaced resource from (ignored in case of ...

---

### `kubernetes_mcp_resources_list`
List Kubernetes resources and objects in the current cluster by providing their apiVersion and kind and optionally the namespace and label selector...

**Parameters:**
- `apiVersion` (string) ✅ — apiVersion of the resources (examples of valid apiVersion are: v1, apps/v1, netw...
- `fieldSelector` (string)  — Optional Kubernetes field selector to filter resources by field values (e.g. 'st...
- `kind` (string) ✅ — kind of the resources (examples of valid kind are: Pod, Service, Deployment, Ing...
- `labelSelector` (string)  — Optional Kubernetes label selector (e.g. 'app=myapp,env=prod' or 'app in (myapp,...
- `namespace` (string)  — Optional Namespace to retrieve the namespaced resources from (ignored in case of...

---

### `kubernetes_mcp_resources_scale`
Get or update the scale of a Kubernetes resource in the current cluster by providing its apiVersion, kind, name, and optionally the namespace. If the scale is set in the tool call, the scale will be u...

**Parameters:**
- `apiVersion` (string) ✅ — apiVersion of the resource (examples of valid apiVersion are apps/v1)
- `kind` (string) ✅ — kind of the resource (examples of valid kind are: StatefulSet, Deployment)
- `name` (string) ✅ — Name of the resource
- `namespace` (string)  — Optional Namespace to get/update the namespaced resource scale from (ignored in ...
- `scale` (integer)  — Optional scale to update the resources scale to. If not provided, will return th...

---

## Playwright (22 tools)

### `playwright_browser_click`
Perform click on a web page

**Parameters:**
- `button` (string)  — Button to click, defaults to left
- `doubleClick` (boolean)  — Whether to perform a double click instead of a single click
- `element` (string)  — Human-readable element description used to obtain permission to interact with th...
- `modifiers` (array)  — Modifier keys to press
- `ref` (string) ✅ — Exact target element reference from the page snapshot

---

### `playwright_browser_close`
Close the page

**Parameters:** *(none)*

---

### `playwright_browser_console_messages`
Returns all console messages

**Parameters:**
- `filename` (string)  — Filename to save the console messages to. If not provided, messages are returned...
- `level` (string) ✅ — Level of the console messages to return. Each level includes the messages of mor...

---

### `playwright_browser_drag`
Perform drag and drop between two elements

**Parameters:**
- `endElement` (string) ✅ — Human-readable target element description used to obtain the permission to inter...
- `endRef` (string) ✅ — Exact target element reference from the page snapshot
- `startElement` (string) ✅ — Human-readable source element description used to obtain the permission to inter...
- `startRef` (string) ✅ — Exact source element reference from the page snapshot

---

### `playwright_browser_evaluate`
Evaluate JavaScript expression on page or element

**Parameters:**
- `element` (string)  — Human-readable element description used to obtain permission to interact with th...
- `function` (string) ✅ — () => { /* code */ } or (element) => { /* code */ } when element is provided
- `ref` (string)  — Exact target element reference from the page snapshot

---

### `playwright_browser_file_upload`
Upload one or multiple files

**Parameters:**
- `paths` (array)  — The absolute paths to the files to upload. Can be single file or multiple files....

---

### `playwright_browser_fill_form`
Fill multiple form fields

**Parameters:**
- `fields` (array) ✅ — Fields to fill in

---

### `playwright_browser_handle_dialog`
Handle a dialog

**Parameters:**
- `accept` (boolean) ✅ — Whether to accept the dialog.
- `promptText` (string)  — The text of the prompt in case of a prompt dialog.

---

### `playwright_browser_hover`
Hover over element on page

**Parameters:**
- `element` (string)  — Human-readable element description used to obtain permission to interact with th...
- `ref` (string) ✅ — Exact target element reference from the page snapshot

---

### `playwright_browser_install`
Install the browser specified in the config. Call this if you get an error about the browser not being installed.

**Parameters:** *(none)*

---

### `playwright_browser_navigate`
Navigate to a URL

**Parameters:**
- `url` (string) ✅ — The URL to navigate to

---

### `playwright_browser_navigate_back`
Go back to the previous page in the history

**Parameters:** *(none)*

---

### `playwright_browser_network_requests`
Returns all network requests since loading the page

**Parameters:**
- `filename` (string)  — Filename to save the network requests to. If not provided, requests are returned...
- `includeStatic` (boolean) ✅ — Whether to include successful static resources like images, fonts, scripts, etc....

---

### `playwright_browser_press_key`
Press a key on the keyboard

**Parameters:**
- `key` (string) ✅ — Name of the key to press or a character to generate, such as `ArrowLeft` or `a`

---

### `playwright_browser_resize`
Resize the browser window

**Parameters:**
- `height` (number) ✅ — Height of the browser window
- `width` (number) ✅ — Width of the browser window

---

### `playwright_browser_run_code`
Run Playwright code snippet

**Parameters:**
- `code` (string) ✅ — A JavaScript function containing Playwright code to execute. It will be invoked ...

---

### `playwright_browser_select_option`
Select an option in a dropdown

**Parameters:**
- `element` (string)  — Human-readable element description used to obtain permission to interact with th...
- `ref` (string) ✅ — Exact target element reference from the page snapshot
- `values` (array) ✅ — Array of values to select in the dropdown. This can be a single value or multipl...

---

### `playwright_browser_snapshot`
Capture accessibility snapshot of the current page, this is better than screenshot

**Parameters:**
- `filename` (string)  — Save snapshot to markdown file instead of returning it in the response.

---

### `playwright_browser_tabs`
List, create, close, or select a browser tab.

**Parameters:**
- `action` (string) ✅ — Operation to perform
- `index` (number)  — Tab index, used for close/select. If omitted for close, current tab is closed.

---

### `playwright_browser_take_screenshot`
Take a screenshot of the current page. You can't perform actions based on the screenshot, use browser_snapshot for actions.

**Parameters:**
- `element` (string)  — Human-readable element description used to obtain permission to screenshot the e...
- `filename` (string)  — File name to save the screenshot to. Defaults to `page-{timestamp}.{png|jpeg}` i...
- `fullPage` (boolean)  — When true, takes a screenshot of the full scrollable page, instead of the curren...
- `ref` (string)  — Exact target element reference from the page snapshot. If not provided, the scre...
- `type` (string) ✅ — Image format for the screenshot. Default is png.

---

### `playwright_browser_type`
Type text into editable element

**Parameters:**
- `element` (string)  — Human-readable element description used to obtain permission to interact with th...
- `ref` (string) ✅ — Exact target element reference from the page snapshot
- `slowly` (boolean)  — Whether to type one character at a time. Useful for triggering key handlers in t...
- `submit` (boolean)  — Whether to submit entered text (press Enter after)
- `text` (string) ✅ — Text to type into the element

---

### `playwright_browser_wait_for`
Wait for text to appear or disappear or a specified time to pass

**Parameters:**
- `text` (string)  — The text to wait for
- `textGone` (string)  — The text to wait for to disappear
- `time` (number)  — The time to wait in seconds

---

## Octocode (13 tools)

### `octocode_githubGetFileContent`
## Read GitHub file content [EXTERNAL: GitHub API]...

**Parameters:**
- `queries` (array) ✅ — Research queries for githubGetFileContent (1-3 queries per call for optimal reso...

---

### `octocode_githubSearchCode`
## Search GitHub code [EXTERNAL: GitHub API]...

**Parameters:**
- `queries` (array) ✅ — Research queries for githubSearchCode (1-3 queries per call for optimal resource...

---

### `octocode_githubSearchPullRequests`
## Search GitHub Pull Requests [EXTERNAL: GitHub API]...

**Parameters:**
- `queries` (array) ✅ — Research queries for githubSearchPullRequests (1-3 queries per call for optimal ...

---

### `octocode_githubSearchRepositories`
## Search GitHub repositories [EXTERNAL: GitHub API]...

**Parameters:**
- `queries` (array) ✅ — Research queries for githubSearchRepositories (1-3 queries per call for optimal ...

---

### `octocode_githubViewRepoStructure`
## Display GitHub repo structure [EXTERNAL: GitHub API]...

**Parameters:**
- `queries` (array) ✅ — Research queries for githubViewRepoStructure (1-3 queries per call for optimal r...

---

### `octocode_localFindFiles`


**Parameters:**
- `queries` (array) ✅ — Queries for localFindFiles (1–5 per call). Review schema before use.

---

### `octocode_localGetFileContent`


**Parameters:**
- `queries` (array) ✅ — Queries for localGetFileContent (1–5 per call). Review schema before use.

---

### `octocode_localSearchCode`


**Parameters:**
- `queries` (array) ✅ — Queries for localSearchCode (1–5 per call). Review schema before use.

---

### `octocode_localViewStructure`


**Parameters:**
- `queries` (array) ✅ — Queries for localViewStructure (1–5 per call). Review schema before use.

---

### `octocode_lspCallHierarchy`


**Parameters:**
- `queries` (array) ✅ — Research queries for lspCallHierarchy (1-3 queries per call for optimal resource...

---

### `octocode_lspFindReferences`


**Parameters:**
- `queries` (array) ✅ — Queries for lspFindReferences (1–5 per call). Review schema before use.

---

### `octocode_lspGotoDefinition`


**Parameters:**
- `queries` (array) ✅ — Queries for lspGotoDefinition (1–5 per call). Review schema before use.

---

### `octocode_packageSearch`
## Find NPM/Python packages [EXTERNAL: npm/PyPI]...

**Parameters:**
- `queries` (array) ✅ — Research queries for packageSearch (1-3 queries per call for optimal resource ma...

---

## Terraform (9 tools)

### `terraform_get_latest_module_version`
Fetches the latest version of a Terraform module from the public registry

**Parameters:**
- `module_name` (string) ✅ — The name of the module, this is usually the service or group of service the user...
- `module_provider` (string) ✅ — The name of the Terraform provider for the module, e.g., 'aws', 'google', 'azure...
- `module_publisher` (string) ✅ — The publisher of the module, e.g., 'hashicorp', 'aws-ia', 'terraform-google-modu...

---

### `terraform_get_latest_provider_version`
Fetches the latest version of a Terraform provider from the public registry

**Parameters:**
- `name` (string) ✅ — The name of the Terraform provider, e.g., 'aws', 'azurerm', 'google', etc.
- `namespace` (string) ✅ — The namespace of the Terraform provider, typically the name of the company, or t...

---

### `terraform_get_module_details`
Fetches up-to-date documentation on how to use a Terraform module. You must call 'search_modules' first to obtain the exact valid and compatible module_id required to use this tool.

**Parameters:**
- `module_id` (string) ✅ — Exact valid and compatible module_id retrieved from search_modules (e.g., 'squar...

---

### `terraform_get_policy_details`
Fetches up-to-date documentation for a specific policy from the Terraform registry. You must call 'search_policies' first to obtain the exact terraform_policy_id required to use this tool.

**Parameters:**
- `terraform_policy_id` (string) ✅ — Matching terraform_policy_id retrieved from the 'search_policies' tool (e.g., 'p...

---

### `terraform_get_provider_capabilities`
Get the capabilities of a Terraform provider including the types of resources, data sources, functions, guides, and other features it supports....

**Parameters:**
- `name` (string) ✅ — The name of the Terraform provider, e.g., 'aws', 'azurerm', 'google', etc.
- `namespace` (string) ✅ — The namespace of the Terraform provider, typically the name of the company, or t...
- `version` (string)  — The version of the provider to analyze (defaults to 'latest')

---

### `terraform_get_provider_details`
Fetches up-to-date documentation for a specific service from a Terraform provider. ...

**Parameters:**
- `provider_doc_id` (string) ✅ — Exact tfprovider-compatible provider_doc_id, (e.g., '8894603', '8906901') retrie...

---

### `terraform_search_modules`
Resolves a Terraform module name to obtain a compatible module_id for the get_module_details tool and returns a list of matching Terraform modules....

**Parameters:**
- `current_offset` (number)  — Current offset for pagination
- `module_query` (string) ✅ — The query to search for Terraform modules.

---

### `terraform_search_policies`
Searches for Terraform policies based on a query string....

**Parameters:**
- `policy_query` (string) ✅ — The query to search for Terraform modules.

---

### `terraform_search_providers`
This tool retrieves a list of potential documents based on the 'service_slug' and 'provider_document_type' provided....

**Parameters:**
- `provider_document_type` (string) ✅ — The type of the document to retrieve,
for general overview of the provider use '...
- `provider_name` (string) ✅ — The name of the Terraform provider to perform the read or deployment operation
- `provider_namespace` (string) ✅ — The publisher of the Terraform provider, typically the name of the company, or t...
- `provider_version` (string)  — The version of the Terraform provider to retrieve in the format 'x.y.z', or 'lat...
- `service_slug` (string) ✅ — The slug of the service you want to deploy or read using the Terraform provider,...

---

## Openmemory (6 tools)

### `openmemory_openmemory_delete`
Delete a memory by identifier

**Parameters:**
- `id` (string) ✅ — Memory identifier to delete
- `user_id` (string)  — Validate ownership

---

### `openmemory_openmemory_get`
Fetch a single memory by identifier

**Parameters:**
- `id` (string) ✅ — Memory identifier to load
- `include_vectors` (boolean)  — Include sector vector metadata
- `user_id` (string)  — Validate ownership against a specific user identifier

---

### `openmemory_openmemory_list`
List recent memories for quick inspection

**Parameters:**
- `limit` (integer)  — Number of memories to return
- `sector` (string)  — Optionally limit to a sector
- `user_id` (string)  — Restrict results to a specific user identifier

---

### `openmemory_openmemory_query`
Query OpenMemory for contextual memories (HSG) and/or temporal facts

**Parameters:**
- `at` (string)  — ISO date string for point-in-time queries (default: now). Queries facts valid at...
- `fact_pattern` (object)  — Fact pattern for temporal queries. Used when type is 'factual' or 'unified'
- `k` (integer)  — Maximum results to return (for HSG queries)
- `min_salience` (number)  — Minimum salience threshold (for HSG queries)
- `query` (string) ✅ — Free-form search text
- `sector` (string)  — Restrict search to a specific sector (for HSG queries)
- `type` (string)  — Query type: 'contextual' for HSG semantic search (default), 'factual' for tempor...
- `user_id` (string)  — Isolate results to a specific user identifier

---

### `openmemory_openmemory_reinforce`
Boost salience for an existing memory

**Parameters:**
- `boost` (number)  — Salience boost amount (default 0.1)
- `id` (string) ✅ — Memory identifier to reinforce

---

### `openmemory_openmemory_store`
Persist new content into OpenMemory (HSG contextual memory and/or temporal facts)

**Parameters:**
- `content` (string) ✅ — Raw memory text to store
- `facts` (array)  — Array of facts to store in temporal graph. Required when type is 'factual' or 'b...
- `metadata` (object)  — Arbitrary metadata blob
- `tags` (array)  — Optional tag list (for HSG storage)
- `type` (string)  — Storage type: 'contextual' for HSG only (default), 'factual' for temporal facts ...
- `user_id` (string)  — Associate the memory with a specific user identifier

---

## Prometheus (6 tools)

### `prometheus_execute_query`
Execute a PromQL instant query against Prometheus

**Parameters:**
- `query` (string) ✅ — 
- `time` (any)  — 

---

### `prometheus_execute_range_query`
Execute a PromQL range query with start time, end time, and step interval

**Parameters:**
- `end` (string) ✅ — 
- `query` (string) ✅ — 
- `start` (string) ✅ — 
- `step` (string) ✅ — 

---

### `prometheus_get_metric_metadata`
Get metadata for a specific metric

**Parameters:**
- `metric` (string) ✅ — 

---

### `prometheus_get_targets`
Get information about all scrape targets

**Parameters:** *(none)*

---

### `prometheus_health_check`
Health check endpoint for container monitoring and status verification

**Parameters:** *(none)*

---

### `prometheus_list_metrics`
List all available metrics in Prometheus with optional pagination support

**Parameters:**
- `filter_pattern` (any)  — 
- `limit` (any)  — 
- `offset` (integer)  — 

---

## Better (4 tools)

### `better_auth_chat`
Engage in interactive conversations and get help with complex queries through Better Auth's specialized AI assistant. ...

**Parameters:**
- `messages` (array) ✅ — Array of messages representing the conversation history

---

### `better_auth_get_file`
Retrieve a specific file from Better Auth's knowledge base by its ID....

**Parameters:**
- `file_id` (string) ✅ — The unique identifier of the file to retrieve (obtain from list_files tool)

---

### `better_auth_list_files`
List all files available in Better Auth's knowledge base....

**Parameters:** *(none)*

---

### `better_auth_search`
Search Better Auth's knowledge base for specific information and documents....

**Parameters:**
- `limit` (integer)  — Maximum number of results to return (1-100, default: 10)
- `mode` (string)  — Search depth: fast (quick results), balanced (moderate depth), or deep (comprehe...
- `query` (string) ✅ — The search query in natural language

---

## Exa (3 tools)

### `exa_mcp_company_research_exa`
Research any company to get business information, news, and insights....

**Parameters:**
- `companyName` (string) ✅ — Name of the company to research
- `numResults` (number)  — Number of search results to return (default: 5)

---

### `exa_mcp_get_code_context_exa`
Find code examples, documentation, and programming solutions. Searches GitHub, Stack Overflow, and official docs....

**Parameters:**
- `query` (string) ✅ — Search query to find relevant context for APIs, Libraries, and SDKs. For example...
- `tokensNum` (number)  — Number of tokens to return (1000-50000). Default is 5000 tokens. Adjust this val...

---

### `exa_mcp_web_search_exa`
Search the web for any topic and get clean, ready-to-use content....

**Parameters:**
- `contextMaxCharacters` (number)  — Maximum characters for context string optimized for LLMs (default: 10000)
- `livecrawl` (string)  — Live crawl mode - 'fallback': use live crawling as backup if cached content unav...
- `numResults` (number)  — Number of search results to return (default: 8)
- `query` (string) ✅ — Websearch query
- `type` (string)  — Search type - 'auto': balanced search (default), 'fast': quick results, 'deep': ...

---

## Solana (3 tools)

### `solana_Ask_Solana_Anchor_Framework_Expert`
Ask questions about developing on Solana with the Anchor Framework.

**Parameters:**
- `question` (string) ✅ — Any question about the Anchor Framework. (how-to, concepts, APIs, SDKs, errors)

---

### `solana_Solana_Documentation_Search`
Search documentation across the Solana ecosystem to get the most up to date information.

**Parameters:**
- `query` (string) ✅ — A search query that will be matched against a corpus of Solana documentation usi...

---

### `solana_Solana_Expert__Ask_For_Help`
A Solana expert that can answer questions about Solana development.

**Parameters:**
- `question` (string) ✅ — A Solana related question. (how-to, concepts, APIs, SDKs, errors)
 Provide as mu...

---

## Ai (2 tools)

### `ai_elements_get_ai_elements_component`
Provides information about an AI Elements component.

**Parameters:** *(none)*

---

### `ai_elements_get_ai_elements_components`
Provides a list of all AI Elements components.

**Parameters:** *(none)*

---

## Graphql (2 tools)

### `graphql_introspect_schema`
Introspect the GraphQL schema, use this tool before doing a query to get the schema information if you do not have it available as a resource already.

**Parameters:**
- `__ignore__` (boolean)  — This does not do anything

---

### `graphql_query_graphql`
Query a GraphQL endpoint with the given query and variables

**Parameters:**
- `query` (string) ✅ — 
- `variables` (string)  — 

---

## Pg (2 tools)

### `pg_aiguide_search_docs`
Search documentation using semantic or keyword search. Supports Tiger Cloud (TimescaleDB) and PostgreSQL.

**Parameters:**
- `limit` (integer) ✅ — The maximum number of matches to return. Default is 10.
- `query` (string) ✅ — The search query. For semantic search, use natural language. For keyword search,...
- `search_type` (string) ✅ — The type of search to perform. "semantic" uses natural language vector similarit...
- `source` (string) ✅ — The documentation source to search. "tiger" for Tiger Cloud and TimescaleDB, "po...
- `version` (string) ✅ — The PostgreSQL major version (ignored when searching "tiger"). Recommended to as...

---

### `pg_aiguide_view_skill`
Retrieve detailed skills for TimescaleDB operations and best practices....

**Parameters:**
- `path` (string) ✅ — A relative path to a file or directory within the skill to view.
If empty, will ...
- `skill_name` (string) ✅ — The name of the skill to browse, or `.` to list all available skills.

---

## Tools (2 tools)

### `tools_list_available_tools`
Get available tools and MCP client config file structure for automated config generation.

**Parameters:** *(none)*

---

### `tools_screenshot_upload`
Upload a screenshot to S3-compatible storage (SeaweedFS) and get a URL for embedding in PRs. Use after browser_take_screenshot to upload captures for visual documentation.

**Parameters:**
- `file_path` (string) ✅ — Path to the screenshot file (e.g., /tmp/playwright-output/screenshot.png)
- `name` (string) ✅ — Descriptive name for the screenshot (e.g., login-form, dashboard-view)
- `pr_number` (integer) ✅ — Pull request number
- `repo` (string) ✅ — Repository name in owner/repo format (e.g., 5dlabs/myproject)

---

