/

* * *

# REST API

- Tier: Free, Premium, Ultimate
- Offering: GitLab.com, GitLab Self-Managed, GitLab Dedicated

Automate your workflows and build integrations with the GitLab REST API:

- Create custom tools to manage your GitLab resources at scale without manual intervention.
- Improve collaboration by integrating GitLab data directly into your applications.
- Manage CI/CD processes across multiple projects with precision.
- Control user access programmatically to maintain consistent permissions across your organization.

The REST API uses standard HTTP methods and JSON data formats
for compatibility with your existing tools and systems.

## Make a REST API request [Permalink](https://docs.gitlab.com/api/rest/\#make-a-rest-api-request "Permalink")

To make a REST API request:

- Submit a request to an API endpoint by using a REST API client.
- The GitLab instance responds to the request. It returns a status code and if applicable, the
requested data. The status code indicates the outcome of the request and is useful when
[troubleshooting](https://docs.gitlab.com/api/rest/troubleshooting/).

A REST API request must start with the root endpoint and the path.

- The root endpoint is the GitLab host name.
- The path must start with `/api/v4` (`v4` represents the API version).

In the following example, the API request retrieves the list of all projects on GitLab host
`gitlab.example.com`:

shell

```shell
curl --request GET \
  --url "https://gitlab.example.com/api/v4/projects"
```

Access to some endpoints require authentication. For more information, see
[Authentication](https://docs.gitlab.com/api/rest/authentication/).

## Rate limits [Permalink](https://docs.gitlab.com/api/rest/\#rate-limits "Permalink")

REST API requests are subject to rate limit settings. These settings reduce the risk of a GitLab
instance being overloaded.

- For details, see [Rate limits](https://docs.gitlab.com/security/rate_limits/).
- For details of the rate limit settings used by GitLab.com, see
[GitLab.com-specific rate limits](https://docs.gitlab.com/user/gitlab_com/#rate-limits-on-gitlabcom).

## Response format [Permalink](https://docs.gitlab.com/api/rest/\#response-format "Permalink")

REST API responses are returned in JSON format. Some API endpoints also support
plain text format. To confirm which content type an endpoint supports, see the
[REST API resources](https://docs.gitlab.com/api/api_resources/).

## Request requirements [Permalink](https://docs.gitlab.com/api/rest/\#request-requirements "Permalink")

Some REST API requests have specific requirements, including the data format and encoding used.

### Request payload [Permalink](https://docs.gitlab.com/api/rest/\#request-payload "Permalink")

API requests can use parameters sent as [query strings](https://en.wikipedia.org/wiki/Query_string)
or as a [payload body](https://datatracker.ietf.org/doc/html/draft-ietf-httpbis-p3-payload-14#section-3.2).
GET requests usually send a query string, while PUT or POST requests usually
send the payload body:

- Query string:





shell







```shell
curl --request POST \
    --url "https://gitlab.example.com/api/v4/projects?name=<example-name>&description=<example-description>"
```

- Request payload (JSON):





shell







```shell
curl --request POST \
    --header "Content-Type: application/json" \
    --data '{"name":"<example-name>", "description":"<example-description>"}' "https://gitlab.example.com/api/v4/projects"
```


URL-encoded query strings have a length limitation. Requests that are too large
result in a `414 Request-URI Too Large` error message. This can be resolved by
using a payload body instead.

### Path parameters [Permalink](https://docs.gitlab.com/api/rest/\#path-parameters "Permalink")

If an endpoint has path parameters, the documentation displays them with a
preceding colon.

For example:

```plaintext
DELETE /projects/:id/share/:group_id
```

The `:id` path parameter needs to be replaced with the project ID, and the
`:group_id` needs to be replaced with the ID of the group. The colons `:`
shouldn’t be included.

The resulting cURL request for a project with ID `5` and a group ID of `17` is then:

shell

```shell
curl --request DELETE \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/5/share/17"
```

Path parameters that are required to be URL-encoded must be followed. If not,
it doesn’t match an API endpoint and responds with a 404. If there’s
something in front of the API (for example, Apache), ensure that it doesn’t decode
the URL-encoded path parameters.

### `id` vs `iid` [Permalink](https://docs.gitlab.com/api/rest/\#id-vs-iid "Permalink")

Some API resources have two similarly-named fields. For example, [issues](https://docs.gitlab.com/api/issues/),
[merge requests](https://docs.gitlab.com/api/merge_requests/), and [project milestones](https://docs.gitlab.com/api/milestones/).
The fields are:

- `id`: ID that is unique across all projects.
- `iid`: Additional, internal ID (displayed in the web UI) that’s unique in the
scope of a single project.

If a resource has both the `iid` field and the `id` field, the `iid` field is
usually used instead of `id` to fetch the resource.

For example, suppose a project with `id: 42` has an issue with `id: 46` and
`iid: 5`. In this case:

- A valid API request to retrieve the issue is `GET /projects/42/issues/5`.
- An invalid API request to retrieve the issue is `GET /projects/42/issues/46`.

Not all resources with the `iid` field are fetched by `iid`. For guidance
regarding which field to use, see the documentation for the specific resource.

### Encoding [Permalink](https://docs.gitlab.com/api/rest/\#encoding "Permalink")

When making a REST API request, some content must be encoded to account for special characters and
data structures.

#### Namespaced paths [Permalink](https://docs.gitlab.com/api/rest/\#namespaced-paths "Permalink")

If using namespaced API requests, make sure that the `NAMESPACE/PROJECT_PATH` is
URL-encoded.

For example, `/` is represented by `%2F`:

```plaintext
GET /api/v4/projects/diaspora%2Fdiaspora
```

A project’s path isn’t necessarily the same as its name. A project’s path is
found in the project’s URL or in the project’s settings, under
**General** \> **Advanced** \> **Change path**.

#### File path, branches, and tags name [Permalink](https://docs.gitlab.com/api/rest/\#file-path-branches-and-tags-name "Permalink")

If a file path, branch or tag contains a `/`, make sure it is URL-encoded.

For example, `/` is represented by `%2F`:

```plaintext
GET /api/v4/projects/1/repository/files/src%2FREADME.md?ref=master
GET /api/v4/projects/1/branches/my%2Fbranch/commits
GET /api/v4/projects/1/repository/tags/my%2Ftag
```

#### Array and hash types [Permalink](https://docs.gitlab.com/api/rest/\#array-and-hash-types "Permalink")

You can request the API with `array` and `hash` types parameters:

##### `array` [Permalink](https://docs.gitlab.com/api/rest/\#array "Permalink")

`import_sources` is a parameter of type `array`:

shell

```shell
curl --request POST \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  -d "import_sources[]=github" \
  -d "import_sources[]=bitbucket" \
  --url "https://gitlab.example.com/api/v4/some_endpoint"
```

##### `hash` [Permalink](https://docs.gitlab.com/api/rest/\#hash "Permalink")

`override_params` is a parameter of type `hash`:

shell

```shell
curl --request POST \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --form "namespace=email" \
  --form "path=impapi" \
  --form "file=@/path/to/somefile.txt" \
  --form "override_params[visibility]=private" \
  --form "override_params[some_other_param]=some_value" \
  --url "https://gitlab.example.com/api/v4/projects/import"
```

##### Array of hashes [Permalink](https://docs.gitlab.com/api/rest/\#array-of-hashes "Permalink")

`variables` is a parameter of type `array` containing hash key/value pairs
`[{ 'key': 'UPLOAD_TO_S3', 'value': 'true' }]`:

shell

```shell
curl --globoff --request POST \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/169/pipeline?ref=master&variables[0][key]=VAR1&variables[0][value]=hello&variables[1][key]=VAR2&variables[1][value]=world"

curl --request POST \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --header "Content-Type: application/json" \
  --data '{ "ref": "master", "variables": [ {"key": "VAR1", "value": "hello"}, {"key": "VAR2", "value": "world"} ] }' \
  --url "https://gitlab.example.com/api/v4/projects/169/pipeline"
```

#### Encoding `+` in ISO 8601 dates [Permalink](https://docs.gitlab.com/api/rest/\#encoding--in-iso-8601-dates "Permalink")

If you need to include a `+` in a query parameter, you may need to use `%2B`
instead, due to a [W3 recommendation](https://www.w3.org/Addressing/URL/4_URI_Recommentations.html)
that causes a `+` to be interpreted as a space. For example, in an ISO 8601 date,
you may want to include a specific time in ISO 8601 format, such as:

```plaintext
2017-10-17T23:11:13.000+05:30
```

The correct encoding for the query parameter would be:

```plaintext
2017-10-17T23:11:13.000%2B05:30
```

## Evaluating a response [Permalink](https://docs.gitlab.com/api/rest/\#evaluating-a-response "Permalink")

In some circumstances the API response may not be as you expect. Issues can include null values and
redirection. If you receive a numeric status code in the response, see
[Status codes](https://docs.gitlab.com/api/rest/troubleshooting/#status-codes).

### `null` vs `false` [Permalink](https://docs.gitlab.com/api/rest/\#null-vs-false "Permalink")

In API responses, some boolean fields can have `null` values.
A `null` boolean has no default value and is neither `true` nor `false`.
GitLab treats `null` values in boolean fields the same as `false`.

In boolean arguments, you should only set `true` or `false` values (not `null`).

### Redirects [Permalink](https://docs.gitlab.com/api/rest/\#redirects "Permalink")

History

- Introduced in GitLab 16.4 [with a flag](https://docs.gitlab.com/administration/feature_flags/) named `api_redirect_moved_projects`. Disabled by default.
- [Generally available](https://gitlab.com/gitlab-org/gitlab/-/merge_requests/137578) in GitLab 16.7. Feature flag `api_redirect_moved_projects` removed.

After [path changes](https://docs.gitlab.com/user/project/repository/#repository-path-changes) the
REST API might respond with a message noting that the endpoint has moved. When this happens, use
the endpoint specified in the `Location` header.

Example of a project moved to a different path:

shell

```shell
curl --request GET \
  --verbose \
  --url "https://gitlab.example.com/api/v4/projects/gitlab-org%2Fold-path-project"
```

The response is:

```plaintext
...
< Location: http://gitlab.example.com/api/v4/projects/81
...
This resource has been moved permanently to https://gitlab.example.com/api/v4/projects/81
```

## Pagination [Permalink](https://docs.gitlab.com/api/rest/\#pagination "Permalink")

GitLab supports the following pagination methods:

- Offset-based pagination. The default method and available on all endpoints except,
in GitLab 16.5 and later, the `users` endpoint.
- Keyset-based pagination. Added to selected endpoints but being
[progressively rolled out](https://gitlab.com/groups/gitlab-org/-/epics/2039).

For large collections, you should use keyset pagination
(when available) instead of offset pagination, for performance reasons.

### Offset-based pagination [Permalink](https://docs.gitlab.com/api/rest/\#offset-based-pagination "Permalink")

History

- The `users` endpoint was [deprecated](https://gitlab.com/gitlab-org/gitlab/-/issues/426547) for offset-based pagination in GitLab 16.5 and is planned for removal in 17.0. This change is a breaking change. Use keyset-based pagination for this endpoint instead.
- The `users` endpoint enforces keyset-based pagination when the number of requested records is greater than 50,000 in GitLab 17.0.

Sometimes, the returned result spans many pages. When listing resources, you can
pass the following parameters:

| Parameter | Description |
| --- | --- |
| `page` | Page number (default: `1`). |
| `per_page` | Number of items to list per page (default: `20`, max: `100`). |

The following example lists 50 [namespaces](https://docs.gitlab.com/api/namespaces/) per page:

shell

```shell
curl --request GET \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/namespaces?per_page=50"
```

There is a [max offset allowed limit](https://docs.gitlab.com/administration/instance_limits/#max-offset-allowed-by-the-rest-api-for-offset-based-pagination) for offset pagination. You can change the limit in GitLab Self-Managed instances.

#### Pagination `Link` header [Permalink](https://docs.gitlab.com/api/rest/\#pagination-link-header "Permalink")

[`Link` headers](https://www.w3.org/wiki/LinkHeader) are returned with each
response. They have `rel` set to `prev`, `next`, `first`, or `last` and contain
the relevant URL. Be sure to use these links instead of generating your own URLs.

For GitLab.com users, [some pagination headers may not be returned](https://docs.gitlab.com/user/gitlab_com/#pagination-response-headers).

The following cURL example limits the output to three items per page
(`per_page=3`), and requests the second page (`page=2`) of [comments](https://docs.gitlab.com/api/notes/)
of the issue with ID `8` which belongs to the project with ID `9`:

shell

```shell
curl --request GET \
  --head \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/9/issues/8/notes?per_page=3&page=2"
```

The response is:

http

```http
HTTP/2 200 OK
cache-control: no-cache
content-length: 1103
content-type: application/json
date: Mon, 18 Jan 2016 09:43:18 GMT
link: <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="prev", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="next", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=1&per_page=3>; rel="first", <https://gitlab.example.com/api/v4/projects/8/issues/8/notes?page=3&per_page=3>; rel="last"
status: 200 OK
vary: Origin
x-next-page: 3
x-page: 2
x-per-page: 3
x-prev-page: 1
x-request-id: 732ad4ee-9870-4866-a199-a9db0cde3c86
x-runtime: 0.108688
x-total: 8
x-total-pages: 3
```

#### Other pagination headers [Permalink](https://docs.gitlab.com/api/rest/\#other-pagination-headers "Permalink")

GitLab also returns the following additional pagination headers:

| Header | Description |
| --- | --- |
| `x-next-page` | The index of the next page. |
| `x-page` | The index of the current page (starting at 1). |
| `x-per-page` | The number of items per page. |
| `x-prev-page` | The index of the previous page. |
| `x-total` | The total number of items. |
| `x-total-pages` | The total number of pages. |

For GitLab.com users, [some pagination headers may not be returned](https://docs.gitlab.com/user/gitlab_com/#pagination-response-headers).

### Keyset-based pagination [Permalink](https://docs.gitlab.com/api/rest/\#keyset-based-pagination "Permalink")

Keyset-pagination allows for more efficient retrieval of pages and - in contrast
to offset-based pagination - runtime is independent of the size of the
collection.

This method is controlled by the following parameters. `order_by` and `sort` are both mandatory.

| Parameter | Required | Description |
| --- | --- | --- |
| `pagination` | yes | `keyset` (to enable keyset pagination). |
| `per_page` | no | Number of items to list per page (default: `20`, max: `100`). |
| `order_by` | yes | Column by which to order by. |
| `sort` | yes | Sort order (`asc` or `desc`) |

The following example lists 50 [projects](https://docs.gitlab.com/api/projects/) per page, ordered
by `id` ascending.

shell

```shell
curl --request GET \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects?pagination=keyset&per_page=50&order_by=id&sort=asc"
```

The response header includes a link to the next page. For example:

http

```http
HTTP/1.1 200 OK
...
Link: <https://gitlab.example.com/api/v4/projects?pagination=keyset&per_page=50&order_by=id&sort=asc&id_after=42>; rel="next"
Status: 200 OK
...
```

The link to the next page contains an additional filter `id_after=42` that
excludes already-retrieved records.

As another example, the following request lists 50 [groups](https://docs.gitlab.com/api/groups/) per page ordered
by `name` ascending using keyset pagination:

shell

```shell
curl --request GET \
  --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/groups?pagination=keyset&per_page=50&order_by=name&sort=asc"
```

The response header includes a link to the next page:

http

```http
HTTP/1.1 200 OK
...
Link: <https://gitlab.example.com/api/v4/groups?pagination=keyset&per_page=50&order_by=name&sort=asc&cursor=eyJuYW1lIjoiRmxpZ2h0anMiLCJpZCI6IjI2IiwiX2tkIjoibiJ9>; rel="next"
Status: 200 OK
...
```

The link to the next page contains an additional filter `cursor=eyJuYW1lIjoiRmxpZ2h0anMiLCJpZCI6IjI2IiwiX2tkIjoibiJ9` that
excludes already-retrieved records.

The `X-NEXT-CURSOR` header contains the cursor value for retrieving the next page’s records,
while the `X-PREV-CURSOR` header contains the cursor value for retrieving the previous page’s,
when available.

The type of filter depends on the
`order_by` option used, and you can have more than one additional filter.

The `Links` header was removed to be aligned with the
[W3C `Link` specification](https://www.w3.org/wiki/LinkHeader). The `Link`
header should be used instead.

When the end of the collection is reached and there are no additional
records to retrieve, the `Link` header is absent and the resulting array is
empty.

You should use only the given link to retrieve the next page instead of
building your own URL. Apart from the headers shown, no additional pagination headers
are exposed.

#### Supported resources [Permalink](https://docs.gitlab.com/api/rest/\#supported-resources "Permalink")

Keyset-based pagination is supported only for selected resources and ordering
options:

| Resource | Options | Availability |
| --- | --- | --- |
| [Group audit events](https://docs.gitlab.com/api/audit_events/#list-all-group-audit-events) | `order_by=id`, `sort=desc` only | Authenticated users only. |
| [Groups](https://docs.gitlab.com/api/groups/#list-groups) | `order_by=name`, `sort=asc` only | Unauthenticated users only. |
| [Instance audit events](https://docs.gitlab.com/api/audit_events/#list-all-instance-audit-events) | `order_by=id`, `sort=desc` only | Authenticated users only. |
| [Package pipelines](https://docs.gitlab.com/api/packages/#list-package-pipelines) | `order_by=id`, `sort=desc` only | Authenticated users only. |
| [Project jobs](https://docs.gitlab.com/api/jobs/#list-all-jobs-for-a-project) | `order_by=id`, `sort=desc` only | Authenticated users only. |
| [Project audit events](https://docs.gitlab.com/api/audit_events/#list-all-project-audit-events) | `order_by=id`, `sort=desc` only | Authenticated users only. |
| [Projects](https://docs.gitlab.com/api/projects/) | `order_by=id` only | Authenticated and unauthenticated users. |
| [Users](https://docs.gitlab.com/api/users/) | `order_by=id`, `order_by=name`, `order_by=username`, `order_by=created_at`, or `order_by=updated_at`. | Authenticated and unauthenticated users. [Introduced](https://gitlab.com/gitlab-org/gitlab/-/issues/419556) in GitLab 16.5. |
| [Registry Repository Tags](https://docs.gitlab.com/api/container_registry/) | `order_by=name`, `sort=asc`, or `sort=desc` only. | Authenticated users only. |
| [List repository tree](https://docs.gitlab.com/api/repositories/#list-all-repository-trees-in-a-project) | N/A | Authenticated and unauthenticated users. [Introduced](https://gitlab.com/gitlab-org/gitlab/-/merge_requests/154897) in GitLab 17.1. |
| [Project issues](https://docs.gitlab.com/api/issues/#list-all-project-issues) | `order_by=created_at`, `order_by=updated_at`, `order_by=title`, `order_by=id`, `order_by=weight`, `order_by=due_date`, `order_by=relative_position`, `sort=asc`, or `sort=desc` only. | Authenticated and unauthenticated users. [Introduced](https://gitlab.com/gitlab-org/gitlab/-/merge_requests/199887/) in GitLab 18.3. |

### Pagination response headers [Permalink](https://docs.gitlab.com/api/rest/\#pagination-response-headers "Permalink")

For performance reasons, if a query returns more than 10,000 records, GitLab
doesn’t return the following headers:

- `x-total`.
- `x-total-pages`.
- `rel="last"``link`

## Versioning and deprecations [Permalink](https://docs.gitlab.com/api/rest/\#versioning-and-deprecations "Permalink")

The REST API version complies with the semantic versioning specification. The major version number
is `4`. Backward-incompatible changes require this version number to change.

- The minor version isn’t explicit, which allows for a stable API endpoint.
- New features are added to the API in the same version number.
- Major API version changes, and removal of entire API versions,
are done in tandem with major GitLab releases.
- All deprecations and changes between versions are noted in the documentation.

The following are excluded from the deprecation process and can be removed at any time without
notice:

- Elements labeled in the [REST API resources](https://docs.gitlab.com/api/api_resources/) as
[experimental or beta](https://docs.gitlab.com/policy/development_stages_support/).
- Fields behind a feature flag and disabled by default.

For GitLab Self-Managed, [reverting](https://docs.gitlab.com/update/convert_to_ee/revert/) from an EE instance to CE causes breaking changes.

Was this page helpful?YesNo

reCAPTCHA

Recaptcha requires verification.

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)

protected by **reCAPTCHA**

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)