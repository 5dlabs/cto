/

* * *

# Repository files API

- Tier: Free, Premium, Ultimate
- Offering: GitLab.com, GitLab Self-Managed, GitLab Dedicated

Use this API to manage [repository files](https://docs.gitlab.com/user/project/repository/).
You can also [configure rate limits](https://docs.gitlab.com/administration/settings/files_api_rate_limits/)
for this API.

## Available scopes for personal access tokens [Permalink](https://docs.gitlab.com/api/repository_files/\#available-scopes-for-personal-access-tokens "Permalink")

[Personal access tokens](https://docs.gitlab.com/user/profile/personal_access_tokens/) support these scopes:

| Scope | Description |
| --- | --- |
| `api` | Allows read-write access to the repository files. |
| `read_api` | Allows read access to the repository files. |
| `read_repository` | Allows read-access to the repository files. |

## Retrieve a file from a repository [Permalink](https://docs.gitlab.com/api/repository_files/\#retrieve-a-file-from-a-repository "Permalink")

Retrieves information about a specified file in a repository. This
includes information like the name, size, and the file contents.
File content is Base64 encoded. You can access this endpoint
without authentication, if the repository is publicly accessible.

For blobs larger than 10 MB, this endpoint has a rate limit of 5 requests per minute.

```plaintext
GET /projects/:id/repository/files/:file_path
```

Supported attributes:

| Attribute | Type | Required | Description |
| --- | --- | --- | --- |
| `file_path` | string | Yes | URL-encoded full path to the file, such as `lib%2Fclass%2Erb`. |
| `id` | integer or string | Yes | ID or [URL-encoded path](https://docs.gitlab.com/api/rest/#namespaced-paths) of the project. |
| `ref` | string | Yes | Name of branch, tag, or commit. Use `HEAD` to automatically use the default branch. |

If successful, returns [`200 OK`](https://docs.gitlab.com/api/rest/troubleshooting/#status-codes) and the following
response attributes:

| Attribute | Type | Description |
| --- | --- | --- |
| `blob_id` | string | Blob SHA. |
| `commit_id` | string | Commit SHA for the file. |
| `content` | string | Base64 encoded file content. |
| `content_sha256` | string | SHA256 hash of the file content. |
| `encoding` | string | Encoding used for the file content. |
| `execute_filemode` | boolean | If `true`, the execute flag is set on the file. |
| `file_name` | string | Name of the file. |
| `file_path` | string | Full path to the file. |
| `last_commit_id` | string | SHA of the last commit that modified this file. |
| `ref` | string | Name of the branch, tag, or commit used. |
| `size` | integer | Size of the file in bytes. |

shell

```shell
curl --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/app%2Fmodels%2Fkey%2Erb?ref=main"
```

If you don’t know the branch name or want to use the default branch, you can use `HEAD` as the
`ref` value. For example:

shell

```shell
curl --header "PRIVATE-TOKEN: " \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/app%2Fmodels%2Fkey%2Erb?ref=HEAD"
```

Example response:

json

```json
{
  "file_name": "key.rb",
  "file_path": "app/models/key.rb",
  "size": 1476,
  "encoding": "base64",
  "content": "IyA9PSBTY2hlbWEgSW5mb3...",
  "content_sha256": "4c294617b60715c1d218e61164a3abd4808a4284cbc30e6728a01ad9aada4481",
  "ref": "main",
  "blob_id": "79f7bbd25901e8334750839545a9bd021f0e4c83",
  "commit_id": "d5a3ff139356ce33e37e73add446f16869741b50",
  "last_commit_id": "570e7b2abdd848b95f2f578043fc23bd6f6fd24d",
  "execute_filemode": false
}
```

### Get file metadata only [Permalink](https://docs.gitlab.com/api/repository_files/\#get-file-metadata-only "Permalink")

You can also use `HEAD` to fetch just file metadata.

```plaintext
HEAD /projects/:id/repository/files/:file_path
```

shell

```shell
curl --head --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/app%2Fmodels%2Fkey%2Erb?ref=main"
```

Example response:

```plaintext
HTTP/1.1 200 OK
...
X-Gitlab-Blob-Id: 79f7bbd25901e8334750839545a9bd021f0e4c83
X-Gitlab-Commit-Id: d5a3ff139356ce33e37e73add446f16869741b50
X-Gitlab-Content-Sha256: 4c294617b60715c1d218e61164a3abd4808a4284cbc30e6728a01ad9aada4481
X-Gitlab-Encoding: base64
X-Gitlab-File-Name: key.rb
X-Gitlab-File-Path: app/models/key.rb
X-Gitlab-Last-Commit-Id: 570e7b2abdd848b95f2f578043fc23bd6f6fd24d
X-Gitlab-Ref: main
X-Gitlab-Size: 1476
X-Gitlab-Execute-Filemode: false
...
```

## Retrieve file blame history from a repository [Permalink](https://docs.gitlab.com/api/repository_files/\#retrieve-file-blame-history-from-a-repository "Permalink")

Retrieves blame history for a specified file in a repository. Each blame range contains lines and their corresponding commit information.

```plaintext
GET /projects/:id/repository/files/:file_path/blame
```

Supported attributes:

| Attribute | Type | Required | Description |
| --- | --- | --- | --- |
| `file_path` | string | Yes | URL-encoded full path to the file, such as `lib%2Fclass%2Erb`. |
| `id` | integer or string | Yes | ID or [URL-encoded path of the project](https://docs.gitlab.com/api/rest/#namespaced-paths). |
| `ref` | string | Yes | Name of branch, tag, or commit. Use `HEAD` to automatically use the default branch. |
| `range` | hash | No | Blame range. |
| `range[end]` | integer | No | Last line of the range to blame. |
| `range[start]` | integer | No | First line of the range to blame. |

If successful, returns [`200 OK`](https://docs.gitlab.com/api/rest/troubleshooting/#status-codes) and the following
response attributes:

| Attribute | Type | Description |
| --- | --- | --- |
| `commit` | object | Commit information for the blame range. |
| `lines` | array | Array of lines for this blame range. |

shell

```shell
curl --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/path%2Fto%2Ffile.rb/blame?ref=main"
```

Example response:

json

```json
[\
  {\
    "commit": {\
      "id": "d42409d56517157c48bf3bd97d3f75974dde19fb",\
      "message": "Add feature\n\nalso fix bug\n",\
      "parent_ids": [\
        "cc6e14f9328fa6d7b5a0d3c30dc2002a3f2a3822"\
      ],\
      "authored_date": "2015-12-18T08:12:22.000Z",\
      "author_name": "John Doe",\
      "author_email": "john.doe@example.com",\
      "committed_date": "2015-12-18T08:12:22.000Z",\
      "committer_name": "John Doe",\
      "committer_email": "john.doe@example.com"\
    },\
    "lines": [\
      "require 'fileutils'",\
      "require 'open3'",\
      ""\
    ]\
  }\
]
```

### Get file blame metadata only [Permalink](https://docs.gitlab.com/api/repository_files/\#get-file-blame-metadata-only "Permalink")

Use the `HEAD` method to return just file blame metadata.

```plaintext
HEAD /projects/:id/repository/files/:file_path/blame
```

shell

```shell
curl --head --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/path%2Fto%2Ffile.rb/blame?ref=main"
```

Example response:

```plaintext
HTTP/1.1 200 OK
...
X-Gitlab-Blob-Id: 79f7bbd25901e8334750839545a9bd021f0e4c83
X-Gitlab-Commit-Id: d5a3ff139356ce33e37e73add446f16869741b50
X-Gitlab-Content-Sha256: 4c294617b60715c1d218e61164a3abd4808a4284cbc30e6728a01ad9aada4481
X-Gitlab-Encoding: base64
X-Gitlab-File-Name: file.rb
X-Gitlab-File-Path: path/to/file.rb
X-Gitlab-Last-Commit-Id: 570e7b2abdd848b95f2f578043fc23bd6f6fd24d
X-Gitlab-Ref: main
X-Gitlab-Size: 1476
X-Gitlab-Execute-Filemode: false
...
```

### Request a blame range [Permalink](https://docs.gitlab.com/api/repository_files/\#request-a-blame-range "Permalink")

To request a blame range, specify `range[start]` and `range[end]` parameters with
the starting and ending line numbers of the file.

shell

```shell
curl --head --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/path%2Fto%2Ffile.rb/blame?ref=main&range[start]=1&range[end]=2"
```

Example response:

json

```json
[\
  {\
    "commit": {\
      "id": "d42409d56517157c48bf3bd97d3f75974dde19fb",\
      "message": "Add feature\n\nalso fix bug\n",\
      "parent_ids": [\
        "cc6e14f9328fa6d7b5a0d3c30dc2002a3f2a3822"\
      ],\
      "authored_date": "2015-12-18T08:12:22.000Z",\
      "author_name": "John Doe",\
      "author_email": "john.doe@example.com",\
      "committed_date": "2015-12-18T08:12:22.000Z",\
      "committer_name": "John Doe",\
      "committer_email": "john.doe@example.com"\
    },\
    "lines": [\
      "require 'fileutils'",\
      "require 'open3'"\
    ]\
  }\
]
```

## Retrieve a raw file from a repository [Permalink](https://docs.gitlab.com/api/repository_files/\#retrieve-a-raw-file-from-a-repository "Permalink")

Retrieves the raw file contents on a specified file in a repository.

```plaintext
GET /projects/:id/repository/files/:file_path/raw
```

Supported attributes:

| Attribute | Type | Required | Description |
| --- | --- | --- | --- |
| `file_path` | string | Yes | URL-encoded full path to the file, such as `lib%2Fclass%2Erb`. |
| `id` | integer or string | Yes | ID or [URL-encoded path of the project](https://docs.gitlab.com/api/rest/#namespaced-paths). |
| `lfs` | boolean | No | If `true`, determines if the response should be Git LFS file contents, rather than the pointer. Ignored if the file is not tracked by Git LFS. Defaults to `false`. |
| `ref` | string | No | Name of branch, tag, or commit. Default is the `HEAD` of the project. |

shell

```shell
curl --header "PRIVATE-TOKEN: <your_access_token>" \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/app%2Fmodels%2Fkey%2Erb/raw?ref=main"
```

Similar to [retrieving a file from a repository](https://docs.gitlab.com/api/repository_files/#retrieve-a-file-from-a-repository),
you can use `HEAD` to get just file metadata.

## Create a file in a repository [Permalink](https://docs.gitlab.com/api/repository_files/\#create-a-file-in-a-repository "Permalink")

History

- Request size and rate limits introduced in GitLab 18.7.

Creates a file in a specified repository. To create multiple files with a single request,
see the [commits API](https://docs.gitlab.com/api/commits/#create-a-commit).

```plaintext
POST /projects/:id/repository/files/:file_path
```

This endpoint is subject to [request size and rate limits](https://docs.gitlab.com/administration/instance_limits/#commits-and-files-api-limits). Requests larger than a default 300 MB limit are rejected. Requests greater than 20 MB are rate limited to 3 requests every 30 seconds.

Supported attributes:

| Attribute | Type | Required | Description |
| --- | --- | --- | --- |
| `branch` | string | Yes | Name of the branch to create. The commit is added to this branch. |
| `commit_message` | string | Yes | Commit message. |
| `content` | string | Yes | The file’s content. |
| `file_path` | string | Yes | URL-encoded full path to the file. For example: `lib%2Fclass%2Erb`. |
| `id` | integer or string | Yes | ID or [URL-encoded path of the project](https://docs.gitlab.com/api/rest/#namespaced-paths). |
| `author_email` | string | No | Commit author’s email address. |
| `author_name` | string | No | Commit author’s name. |
| `encoding` | string | No | Change encoding to `base64`. Default is `text`. |
| `execute_filemode` | boolean | No | If `true`, enables the `execute` flag on the file. If `false`, disables the `execute` flag on the file. |
| `start_branch` | string | No | Name of the base branch to create the branch from. |

If successful, returns [`201 Created`](https://docs.gitlab.com/api/rest/troubleshooting/#status-codes) and the following
response attributes:

| Attribute | Type | Description |
| --- | --- | --- |
| `branch` | string | Name of the branch the file was created in. |
| `file_path` | string | Path to the created file. |

shell

```shell
curl --request POST \
  --header 'PRIVATE-TOKEN: <your_access_token>' \
  --header "Content-Type: application/json" \
  --data '{"branch": "main", "author_email": "author@example.com", "author_name": "Firstname Lastname",
            "content": "some content", "commit_message": "create a new file"}' \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/app%2Fproject%2Erb"
```

Example response:

json

```json
{
  "file_path": "app/project.rb",
  "branch": "main"
}
```

## Update a file in a repository [Permalink](https://docs.gitlab.com/api/repository_files/\#update-a-file-in-a-repository "Permalink")

History

- Request size and rate limits introduced in GitLab 18.7.

Updates a specified file in a repository. To update multiple files with a single request,
see the [commits API](https://docs.gitlab.com/api/commits/#create-a-commit).

```plaintext
PUT /projects/:id/repository/files/:file_path
```

This endpoint is subject to [request size and rate limits](https://docs.gitlab.com/administration/instance_limits/#commits-and-files-api-limits). Requests larger than a default 300 MB limit are rejected. Requests greater than 20 MB are rate limited to 3 requests every 30 seconds.

Supported attributes:

| Attribute | Type | Required | Description |
| --- | --- | --- | --- |
| `branch` | string | Yes | Name of the branch to create. The commit is added to this branch. |
| `commit_message` | string | Yes | Commit message. |
| `content` | string | Yes | File’s content. |
| `file_path` | string | Yes | URL-encoded full path to the file. For example: `lib%2Fclass%2Erb`. |
| `id` | integer or string | Yes | ID or [URL-encoded path of the project](https://docs.gitlab.com/api/rest/#namespaced-paths) |
| `author_email` | string | No | Commit author’s email address. |
| `author_name` | string | No | Commit author’s name. |
| `encoding` | string | No | Change encoding to `base64`. Default is `text`. |
| `execute_filemode` | boolean | No | If `true`, enables the `execute` flag on the file. If `false`, disables the `execute` flag on the file. |
| `last_commit_id` | string | No | Last known file commit ID. |
| `start_branch` | string | No | Name of the base branch to create the branch from. |

If successful, returns [`200 OK`](https://docs.gitlab.com/api/rest/troubleshooting/#status-codes) and the following
response attributes:

| Attribute | Type | Description |
| --- | --- | --- |
| `branch` | string | Name of the branch the file was updated in. |
| `file_path` | string | Path to the updated file. |

shell

```shell
curl --request PUT \
  --header 'PRIVATE-TOKEN: <your_access_token>' \
  --header "Content-Type: application/json" \
  --data '{"branch": "main", "author_email": "author@example.com", "author_name": "Firstname Lastname",
       "content": "some content", "commit_message": "update file"}' \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/app%2Fproject%2Erb"
```

Example response:

json

```json
{
  "file_path": "app/project.rb",
  "branch": "main"
}
```

If the commit fails for any reason, the API returns a `400 Bad Request` error with a non-specific
error message. Possible causes for a failed commit include:

- The `file_path` contained `/../` (attempted directory traversal).
- The commit was empty: new file contents were identical to the current file contents.
- Someone updated the branch with `git push` while the file edit was in progress.

[GitLab Shell](https://gitlab.com/gitlab-org/gitlab-shell/) has a Boolean return code, preventing GitLab from specifying the error.

## Delete a file in a repository [Permalink](https://docs.gitlab.com/api/repository_files/\#delete-a-file-in-a-repository "Permalink")

Deletes a specified file in a repository. To delete multiple files with a single request,
see the [commits API](https://docs.gitlab.com/api/commits/#create-a-commit).

```plaintext
DELETE /projects/:id/repository/files/:file_path
```

Supported attributes:

| Attribute | Type | Required | Description |
| --- | --- | --- | --- |
| `branch` | string | Yes | Name of the branch to create. The commit is added to this branch. |
| `commit_message` | string | Yes | Commit message. |
| `file_path` | string | Yes | URL-encoded full path to the file. For example: `lib%2Fclass%2Erb`. |
| `id` | integer or string | Yes | ID or [URL-encoded path of the project](https://docs.gitlab.com/api/rest/#namespaced-paths). |
| `author_email` | string | No | Commit author’s email address. |
| `author_name` | string | No | Commit author’s name. |
| `last_commit_id` | string | No | Last known file commit ID. |
| `start_branch` | string | No | Name of the base branch to create the branch from. |

If successful, returns [`200 OK`](https://docs.gitlab.com/api/rest/troubleshooting/#status-codes).

shell

```shell
curl --request DELETE \
  --header 'PRIVATE-TOKEN: <your_access_token>' \
  --header "Content-Type: application/json" \
  --data '{"branch": "main", "author_email": "author@example.com", "author_name": "Firstname Lastname",
       "commit_message": "delete file"}' \
  --url "https://gitlab.example.com/api/v4/projects/13083/repository/files/app%2Fproject%2Erb"
```

Was this page helpful?YesNo

reCAPTCHA

Recaptcha requires verification.

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)

protected by **reCAPTCHA**

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)