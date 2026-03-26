[Snippets](https://gitlab.com/explore/snippets) [Groups](https://gitlab.com/explore/groups) [Projects](https://gitlab.com/explore/projects)

### Files

Search files (\*.vue, \*.rb...)

[Provide feedback](https://gitlab.com/gitlab-org/gitlab/-/issues/581271)

main


Select Git revision


- Selected


- main
default
protected


- Branches
19

- docs-update-alerts

- add-jq-to-dockerfile

- huh-ugh-v2

- renovate/go-version

- ccorona2/wi-update

- feature/api-multipart-form-upload

- rename-to-reopen

- ccorona2/wi-create

- add-release-issue-notifications

- 7707-convert-mr-checkout-tests-to-newer-stack-git-tests

- azubov/2096-provide-configurable-duo-cli-path

- andunn-8216-fix-array-param-serialization

- jmc-allow-work-items

- renovate/github.com-charmbracelet-glamour-2.x

- tv/2026-03/glamour-v2

- tv/2026-03/mr-note

- include-file-context-in-mr-comments-in-non-tty

- glab-stack-worktree-support

- credential-helper-wrapper-detection


- Tags
20

- v1.89.0

protected

- v1.88.0

protected

- v1.87.0

protected

- v1.86.0

protected

- v1.85.3

protected

- v1.85.2

protected

- v1.85.1

protected

- v1.85.0

protected

- v1.84.0

protected

- v1.83.0

protected

- v1.82.0

protected

- v1.81.0

protected

- v1.80.4

protected

- v1.80.3

protected

- v1.80.2

protected

- v1.80.1

protected

- v1.80.0

protected

- v1.79.0

protected

- v1.78.3

protected

- v1.78.2

protected


40 results


# README.md

Find file
[Blame](https://gitlab.com/gitlab-org/cli/-/blame/main/README.md)

Edit


- [Edit single file\\
Edit this file only.](https://gitlab.com/gitlab-org/cli/-/edit/main/README.md)

File actions


- Find file `t`
- [Blame](https://gitlab.com/gitlab-org/cli/-/blame/main/README.md)
- Copy permalink `y`
- Copy contents

- [Open raw](https://gitlab.com/gitlab-org/cli/-/raw/main/README.md)
- [Download](https://gitlab.com/gitlab-org/cli/-/raw/main/README.md?inline=false)

[![Timo Furrer's avatar](https://gitlab.com/uploads/-/system/user/avatar/12689435/avatar.png?width=64)](https://gitlab.com/timofurrer)

[feat(ci auto login): move from experimental to GA](https://gitlab.com/gitlab-org/cli/-/commit/ceb51ebd03ec0e92f863c1afec4041b035188f0c)

Most recent commit.


[Timo Furrer](https://gitlab.com/timofurrer) authored 2 weeks ago

```
This change set moves the CI auto-login feature out of experimental to
general availability. We haven't seen any issues with it and it's opt-in
which wouldn't disrupt existing users.

In the future we can consider turning the feature on by default, but in
some edge-cases this might be a breaking change. The workaround would be
to just disable the feature in those scenarios.

Closes #8071
```

ceb51ebd

[History](https://gitlab.com/gitlab-org/cli/-/commits/main/README.md)

[![Timo Furrer's avatar](https://gitlab.com/uploads/-/system/user/avatar/12689435/avatar.png?width=64)](https://gitlab.com/timofurrer)[ceb51ebd](https://gitlab.com/gitlab-org/cli/-/commit/ceb51ebd03ec0e92f863c1afec4041b035188f0c) 2 weeks ago

[History](https://gitlab.com/gitlab-org/cli/-/commits/main/README.md)

[Code owners](https://gitlab.com/gitlab-org/cli/-/blob/main/.gitlab/CODEOWNERS)

1
Show all


[Amy Qualls](https://gitlab.com/aqualls)

**README.md** 29.16 KiB

Code Preview

Table of contents


- [GLab](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#glab)
- [Table of contents](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#table-of-contents)
- [Requirements](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#requirements)
- [Usage](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#usage)
- [Core commands](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#core-commands)
- [GitLab Duo for the CLI](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#gitlab-duo-for-the-cli)
- [Demo](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#demo)
- [Documentation](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#documentation)
- [Installation](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#installation)
- [Homebrew](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#homebrew)
- [Other installation methods](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#other-installation-methods)
- [Building from source](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#building-from-source)
- [Prerequisites for building from source](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#prerequisites-for-building-from-source)
- [Authentication](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#authentication)
- [OAuth (GitLab.com)](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#oauth-gitlabcom)
- [OAuth (GitLab Self-Managed, GitLab Dedicated)](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#oauth-gitlab-self-managed-gitlab-dedicated)
- [Personal access token](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#personal-access-token)
- [CI Job Token](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#ci-job-token)
- [Auto-login](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#auto-login)
- [Manual login](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#manual-login)
- [Configuration](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration)
- [Configuration Levels](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration-levels)
- [Configuration Search Order](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration-search-order)
- [Configuration File Locations](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration-file-locations)
- [Configure glab to use your GitLab Self-Managed or GitLab Dedicated instance](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configure-glab-to-use-your-gitlab-self-managed-or-gitlab-dedicated-instance)
- [Configure glab to use mTLS certificates](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configure-glab-to-use-mtls-certificates)
- [Configure glab to use self-signed certificates](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configure-glab-to-use-self-signed-certificates)
- [Environment variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#environment-variables)
- [GitLab access variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#gitlab-access-variables)
- [glab configuration variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#glab-configuration-variables)
- [Other variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#other-variables)
- [Variable deprecation](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#variable-deprecation)
- [Token and environment variable precedence](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#token-and-environment-variable-precedence)
- [Debugging](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#debugging)
- [Troubleshooting](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#troubleshooting)
- [Issues](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#issues)
- [Contributing](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#contributing)
- [Versioning](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#versioning)
- [Classify version changes](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#classify-version-changes)
- [Compatibility](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#compatibility)
- [Inspiration](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#inspiration)

[Open raw](https://gitlab.com/gitlab-org/cli/-/raw/main/README.md "Open raw")[Download](https://gitlab.com/gitlab-org/cli/-/raw/main/README.md?inline=false "Download")

# GLab [Link to heading 'GLab'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#glab)

![GLab](https://gitlab.com/gitlab-org/cli/-/raw/main/docs/source/img/glab-logo.png)

GLab is an open source GitLab CLI tool. It brings GitLab to your terminal, next to where you are already working with `git` and your code, without switching between windows and browser tabs. While it's powerful for issues and merge requests, `glab` does even more:

- View, manage, and retry CI/CD pipelines directly from your CLI.
- Create changelogs.
- Create and manage releases.
- Ask GitLab Duo Chat (Classic) questions about Git.
- Manage GitLab agents for Kubernetes.

`glab` is available for repositories hosted on GitLab.com, GitLab Dedicated, and GitLab Self-Managed. It supports multiple authenticated GitLab instances, and automatically detects the authenticated hostname from the remotes available in your working Git directory.

![command example](https://gitlab.com/gitlab-org/cli/-/raw/main/docs/source/img/glabgettingstarted.gif)

## Table of contents [Link to heading 'Table of contents'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#table-of-contents)

- [Requirements](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#requirements)
- [Usage](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#usage)  - [Core commands](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#core-commands)
  - [GitLab Duo for the CLI](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#gitlab-duo-for-the-cli)
- [Demo](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#demo)
- [Documentation](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#documentation)
- [Installation](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#installation)  - [Homebrew](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#homebrew)
  - [Other installation methods](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#other-installation-methods)
  - [Building from source](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#building-from-source)    - [Prerequisites for building from source](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#prerequisites-for-building-from-source)
- [Authentication](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#authentication)  - [OAuth (GitLab.com)](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#oauth-gitlabcom)
  - [OAuth (GitLab Self-Managed, GitLab Dedicated)](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#oauth-gitlab-self-managed-gitlab-dedicated)
  - [Personal access token](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#personal-access-token)
  - [CI Job Token](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#ci-job-token)
- [Configuration](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration)  - [Configure `glab` to use your GitLab Self-Managed or GitLab Dedicated instance](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configure-glab-to-use-your-gitlab-self-managed-or-gitlab-dedicated-instance)
  - [Configure `glab` to use mTLS certificates](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configure-glab-to-use-mtls-certificates)
  - [Configure `glab` to use self-signed certificates](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configure-glab-to-use-self-signed-certificates)
- [Environment variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#environment-variables)  - [GitLab access variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#gitlab-access-variables)
  - [`glab` configuration variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#glab-configuration-variables)
  - [Other variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#other-variables)
  - [Token and environment variable precedence](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#token-and-environment-variable-precedence)
  - [Debugging](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#debugging)
- [Troubleshooting](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#troubleshooting)
- [Issues](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#issues)
- [Contributing](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#contributing)  - [Versioning](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#versioning)
  - [Classify version changes](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#classify-version-changes)
  - [Compatibility](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#compatibility)
- [Inspiration](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#inspiration)

## Requirements [Link to heading 'Requirements'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#requirements)

`glab` officially supports GitLab versions 16.0 and later. Certain commands might require
more recent versions. While many commands might work properly in GitLab versions
15.x and earlier, no support is provided for these versions.

## Usage [Link to heading 'Usage'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#usage)

To get started with `glab`:

1. Follow the [installation instructions](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#installation) appropriate for your operating system.
2. [Authenticate](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#authentication) into your instance of GitLab.
3. Optional. Configure `glab` further to meet your needs:

   - 1Password users can configure it to [authenticate to `glab`](https://developer.1password.com/docs/cli/shell-plugins/gitlab/).
   - Set any needed global, per-project, or per-host [configuration](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration).
   - Set any needed [environment variables](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#environment-variables).

You're ready!

### Core commands [Link to heading 'Core commands'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#core-commands)

Run `glab --help` to view a list of core commands in your terminal.

- [`glab alias`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/alias): Create, list, and delete aliases.
- [`glab api`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/api): Make authenticated requests to the GitLab API.
- [`glab auth`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/auth): Manage the authentication state of the CLI.
- [`glab changelog`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/changelog): Interact with the changelog API.
- [`glab check-update`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/check-update): Check for updates to the CLI.
- [`glab ci`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/ci): Work with GitLab CI/CD pipelines and jobs.
- [`glab cluster`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/cluster): Manage GitLab agents for Kubernetes and their clusters.
- [`glab completion`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/completion): Generate shell completion scripts.
- [`glab config`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/config): Set and get CLI settings.
- [`glab deploy-key`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/deploy-key): Manage deploy keys.
- [`glab duo`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/duo): Generate terminal commands from natural language.
- [`glab gpg-key`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/gpg-key): Manage GPG keys registered with your GitLab account.
- [`glab incident`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/incident): Work with GitLab incidents.
- [`glab issue`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/issue): Work with GitLab issues.
- [`glab iteration`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/iteration): Retrieve iteration information.
- [`glab job`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/job): Work with GitLab CI/CD jobs.
- [`glab label`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/label): Manage labels for your project.
- [`glab mcp`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/mcp): Work with a Model Context Protocol (MCP) server. (EXPERIMENTAL)
- [`glab milestone`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/milestone): Manage group or project milestones.
- [`glab mr`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/mr): Create, view, and manage merge requests.
- [`glab opentofu`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/opentofu): Work with the OpenTofu or Terraform integration.
- [`glab release`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/release): Manage GitLab releases.
- [`glab repo`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/repo): Work with GitLab repositories and projects.
- [`glab schedule`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/schedule): Work with GitLab CI/CD schedules.
- [`glab securefile`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/securefile): Manage secure files for a project.
- [`glab snippet`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/snippet): Create, view and manage snippets.
- [`glab ssh-key`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/ssh-key): Manage SSH keys registered with your GitLab account.
- [`glab stack`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/stack): Create, manage, and work with stacked diffs.
- [`glab token`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/token): Manage personal, project, or group tokens.
- [`glab user`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/user): Interact with a GitLab user account.
- [`glab variable`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/variable): Manage variables for a GitLab project or group.
- [`glab version`](https://gitlab.com/gitlab-org/cli/-/tree/main/docs/source/version): Show version information for the CLI.

Commands follow this pattern:

```shell
glab <command> <subcommand> [flags]
```

Many core commands also have sub-commands. Some examples:

- List merge requests assigned to you: `glab mr list --assignee=@me`
- List review requests for you: `glab mr list --reviewer=@me`
- Approve a merge request: `glab mr approve 235`
- Create an issue, and add milestone, title, and label: `glab issue create -m release-2.0.0 -t "My title here" --label important`

### GitLab Duo for the CLI [Link to heading 'GitLab Duo for the CLI'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#gitlab-duo-for-the-cli)

The GitLab CLI also provides support for GitLab Duo AI/ML powered features. These include:

- [`glab duo ask`](https://gitlab.com/gitlab-org/cli/-/blob/main/docs/source/duo/ask.md)

Use `glab duo ask` to ask GitLab Duo (Classic) questions about `git` commands. It can help you
remember a command you forgot, or provide suggestions on how to run commands to perform other tasks.

To interact with the GitLab Duo Agent Platform, use the [GitLab Duo CLI](https://docs.gitlab.com/user/gitlab_duo_cli/).

A unified experience is proposed in [issue 585937](https://gitlab.com/gitlab-org/gitlab/-/work_items/585937 "Unified CLI Experience - Duo CLI and glab Consolidation").

## Demo [Link to heading 'Demo'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#demo)

[![asciicast](https://user-content.gitlab-static.net/2da75374c109f218b13a52292b2bc2640bf25787/68747470733a2f2f61736369696e656d612e6f72672f612f3336383632322e737667)](https://asciinema.org/a/368622)

## Documentation [Link to heading 'Documentation'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#documentation)

Read the [documentation](https://gitlab.com/gitlab-org/cli/-/blob/main/docs/source/_index.md) for usage instructions or check out `glab help`.

## Installation [Link to heading 'Installation'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#installation)

Download a binary suitable for your OS at the [releases page](https://gitlab.com/gitlab-org/cli/-/releases).
Other installation methods depend on your operating system.

### Homebrew [Link to heading 'Homebrew'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#homebrew)

Homebrew is the officially supported package manager for macOS, Linux, and Windows (through [Windows Subsystem for Linux](https://learn.microsoft.com/en-us/windows/wsl/install))

- Homebrew
  - Install with: `brew install glab`
  - Update with: `brew upgrade glab`

### Other installation methods [Link to heading 'Other installation methods'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#other-installation-methods)

Other options to install the GitLab CLI that may not be officially supported or are maintained by the community are [also available](https://gitlab.com/gitlab-org/cli/-/blob/main/docs/installation_options.md).

### Building from source [Link to heading 'Building from source'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#building-from-source)

If a supported binary for your OS is not found at the [releases page](https://gitlab.com/gitlab-org/cli/-/releases), you can build from source:

#### Prerequisites for building from source [Link to heading 'Prerequisites for building from source'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#prerequisites-for-building-from-source)

- `make`
- Go version as defined by [`main/go.mod`](https://gitlab.com/gitlab-org/cli/-/blob/main/go.mod?ref_type=heads#L3)

To build from source:

1. Run `go version` to verify that you have the minimum required Go version.
If Go is not installed, see [Download and install](https://go.dev/doc/install).
2. Clone the repository: `git clone https://gitlab.com/gitlab-org/cli.git`
3. Build the binary: `make build`
4. Install `glab` in `$GOPATH/bin`: `make install`
5. Optional. If `$GOPATH/bin` or `$GOBIN` is not in your `$PATH`,
run `export PATH=$PWD/bin:$PATH`.
6. Confirm the installation: `glab version`

## Authentication [Link to heading 'Authentication'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#authentication)

When running `glab auth login` interactively inside a Git repository, `glab` automatically
detects GitLab instances from your Git remotes and presents them as options. This saves you
from having to manually type the hostname.

### OAuth (GitLab.com) [Link to heading 'OAuth (GitLab.com)'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#oauth-gitlabcom)

To authenticate your installation of `glab` with an OAuth application connected to GitLab.com:

1. Start interactive setup with `glab auth login`.
2. For the GitLab instance you want to sign in to, select **GitLab.com**.
3. For the login method, select **Web**. This selection launches your web browser
to request authorization for the GitLab CLI to use your GitLab.com account.
4. Select **Authorize**.
5. Complete the authentication process in your terminal, selecting the appropriate options for your needs.

### OAuth (GitLab Self-Managed, GitLab Dedicated) [Link to heading 'OAuth (GitLab Self-Managed, GitLab Dedicated)'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#oauth-gitlab-self-managed-gitlab-dedicated)

Prerequisites:

- You've created an OAuth application at the user, group, or instance level, and you
have its application ID. For instructions, see how to configure GitLab
[as an OAuth 2.0 authentication identity provider](https://docs.gitlab.com/integration/oauth_provider/)
in the GitLab documentation.
- Your OAuth application is configured with these parameters:
  - **Redirect URI** is `http://localhost:7171/auth/redirect`.
  - **Confidential** is not selected.
  - **Scopes** are `openid`, `profile`, `read_user`, `write_repository`, and `api`.

To authenticate your installation of `glab` with an OAuth application connected
to your GitLab Self-Managed or GitLab Dedicated instance:

1. Store the application ID with `glab config set client_id <CLIENT_ID> --host <HOSTNAME>`.
For `<CLIENT_ID>`, provide your application ID.
2. Start interactive setup with `glab auth login --hostname <HOSTNAME>`.
3. For the login method, select **Web**. This selection launches your web browser
to request authorization for the GitLab CLI to use your GitLab Self-Managed or GitLab Dedicated account.
4. Select **Authorize**.
5. Complete the authentication process in your terminal, selecting the appropriate options for your needs.

### Personal access token [Link to heading 'Personal access token'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#personal-access-token)

To authenticate your installation of `glab` with a personal access token:

1. Get a GitLab personal access token with at least the `api`
and `write_repository` scopes. Use the method appropriate for your instance:

   - For GitLab.com, create one at the [personal access tokens](https://gitlab.com/-/user_settings/personal_access_tokens?scopes=api%2Cwrite_repository) page.
   - For GitLab Self-Managed and GitLab Dedicated, visit `https://gitlab.example.com/-/user_settings/personal_access_tokens?scopes=api,write_repository`,
     modifying `gitlab.example.com` to match the domain name of your instance.
2. Start interactive setup: `glab auth login`
3. Authenticate with the method appropriate for your GitLab instance:
   - For GitLab SaaS, authenticate against `gitlab.com` by reading the token
     from a file: `glab auth login --stdin < myaccesstoken.txt`
   - For GitLab Self-Managed and GitLab Dedicated, authenticate by reading from a file:
     `glab auth login --hostname gitlab.example.com --stdin < myaccesstoken.txt`. This will allow you to perform
     authenticated `glab` commands against your instance when you are in a Git repository with a remote
     matching your instance's host. Alternatively, set `GITLAB_HOST` to direct your command to your instance.
   - Authenticate with token and hostname: `glab auth login --hostname gitlab.example.org --token xxxxx`
     Not recommended for shared environments.
   - Credentials are stored in the global [configuration file](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration).

### CI Job Token [Link to heading 'CI Job Token'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#ci-job-token)

To authenticate your installation of `glab` with a CI job token, the `glab` command must be run in a GitLab CI job.
The token is automatically provided by the GitLab Runner via the `CI_JOB_TOKEN` environment variable.

Endpoints allowing the use of the CI job token are listed in the
[GitLab documentation](https://docs.gitlab.com/ci/jobs/ci_job_token/#job-token-access).

#### Auto-login [Link to heading 'Auto-login'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#auto-login)

To enable CI auto-login, set `GLAB_ENABLE_CI_AUTOLOGIN=true`. When enabled, `glab` automatically
detects if it's running in a GitLab CI job and uses the predefined CI/CD variables to sign in.

```shell
GLAB_ENABLE_CI_AUTOLOGIN=true glab release list -R $CI_PROJECT_PATH
```

#### Manual login [Link to heading 'Manual login'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#manual-login)

Example:

```shell
glab auth login --job-token $CI_JOB_TOKEN --hostname $CI_SERVER_HOST --api-protocol $CI_SERVER_PROTOCOL
GITLAB_HOST=$CI_SERVER_URL glab release list -R $CI_PROJECT_PATH
```

## Configuration [Link to heading 'Configuration'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#configuration)

By default, `glab` follows the
[XDG Base Directory Spec](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html),
which means it searches for configuration files in multiple locations with proper precedence.

### Configuration Levels [Link to heading 'Configuration Levels'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#configuration-levels)

Configure `glab` at different levels: system-wide, globally (per-user), locally (per-repository), or per host:

- **System-wide** (for all users): Place configuration at `/etc/xdg/glab-cli/config.yml` (or `$XDG_CONFIG_DIRS/glab-cli/config.yml`).
  - Useful for Linux distributions and system administrators to provide default configurations.
  - User configurations will override system-wide settings.
- **Globally** (per-user): run `glab config set --global editor vim`.
  - The global configuration file is available at `~/.config/glab-cli/config.yml` (or `$XDG_CONFIG_HOME/glab-cli/config.yml`).
  - To override this location, set the `GLAB_CONFIG_DIR` environment variable.
- **The current repository**: run `glab config set editor vim` in any folder in a Git repository.
  - The local configuration file is available at `.git/glab-cli/config.yml` in the current working Git directory.
- **Per host**: run `glab config set editor vim --host gitlab.example.org`, changing
the `--host` parameter to meet your needs.
  - Per-host configuration info is always stored in the global configuration file, with or without the `global` flag.

### Configuration Search Order [Link to heading 'Configuration Search Order'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#configuration-search-order)

When `glab` looks for configuration files, it searches in this order (highest priority first):

1. `$GLAB_CONFIG_DIR/config.yml` (if `GLAB_CONFIG_DIR` is set)
2. `~/.config/glab-cli/config.yml` (legacy location, for backward compatibility)
3. `$XDG_CONFIG_HOME/glab-cli/config.yml` (platform-specific XDG location)
4. `$XDG_CONFIG_DIRS/glab-cli/config.yml` (system-wide configs, default: `/etc/xdg/glab-cli/config.yml`)

The first configuration file found is used.

#### Configuration File Locations [Link to heading 'Configuration File Locations'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#configuration-file-locations)

**For backward compatibility**, `glab` checks `~/.config/glab-cli/config.yml` first on all platforms.
If no legacy config exists, `glab` uses platform-specific XDG Base Directory locations:

- **Linux**: `~/.config/glab-cli/config.yml` (XDG\_CONFIG\_HOME)
- **macOS**: `~/Library/Application Support/glab-cli/config.yml` (XDG\_CONFIG\_HOME)
- **Windows**: `%APPDATA%\glab-cli\config.yml` (XDG\_CONFIG\_HOME)

**Note**: If you have config files in both the legacy location (`~/.config/glab-cli/config.yml`)
and the platform-specific XDG location, `glab` will use the legacy location and display a warning.
Consider consolidating to one location to avoid confusion.

### Configure `glab` to use your GitLab Self-Managed or GitLab Dedicated instance [Link to heading 'Configure glab to use your GitLab Self-Managed or GitLab Dedicated instance'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#configure-glab-to-use-your-gitlab-self-managed-or-gitlab-dedicated-instance)

When outside a Git repository, `glab` uses `gitlab.com` by default. For `glab` to default
to your GitLab Self-Managed or GitLab Dedicated instance when you are not in a Git repository, change the host
configuration settings. Use this command, changing `gitlab.example.com` to the domain name
of your instance:

```shell
glab config set -g host gitlab.example.com
```

Setting this configuration enables you to perform commands outside a Git repository while
using your GitLab Self-Managed or GitLab Dedicated instance. For example:

- `glab repo clone group/project`
- `glab issue list -R group/project`

If you don't set a default domain name, you can declare one for the current command with
the `GITLAB_HOST` environment variable, like this:

- `GITLAB_HOST=gitlab.example.com glab repo clone group/project`
- `GITLAB_HOST=gitlab.example.com glab issue list -R group/project`

When inside a Git repository `glab` will use that repository's GitLab host by default. For example `glab issue list`
will list all issues of the current directory's Git repository.

### Configure `glab` to use mTLS certificates [Link to heading 'Configure glab to use mTLS certificates'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#configure-glab-to-use-mtls-certificates)

To use a mutual TLS (Mutual Transport Layer Security) certificate with `glab`, edit your global
configuration file (`~/.config/glab-cli/config.yml`) to provide connection information:

```yaml
hosts:
    git.your-domain.com:
        api_protocol: https
        api_host: git.your-domain.com
        token: xxxxxxxxxxxxxxxxxxxxxxxxx
        client_cert: /path/to/client.crt
        client_key: /path/to/client.key
        ca_cert: /path/to/ca-chain.pem
```

- `ca_cert` is optional for mTLS support if you use a publicly signed server certificate.
- `token` is not required if you use a different authentication method.

### Configure `glab` to use self-signed certificates [Link to heading 'Configure glab to use self-signed certificates'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#configure-glab-to-use-self-signed-certificates)

To configure the GitLab CLI to support GitLab Self-Managed and GitLab Dedicated instances with
self-signed certificates, either:

- Disable TLS verification with:



```shell
glab config set skip_tls_verify true --host gitlab.example.com
```

- Add the path to the self signed CA:



```shell
glab config set ca_cert /path/to/server.pem --host gitlab.example.com
```


## Environment variables [Link to heading 'Environment variables'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#environment-variables)

### GitLab access variables [Link to heading 'GitLab access variables'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#gitlab-access-variables)

| Token name | In `config.yml` | Default value if [not set](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration) | Description |
| --- | --- | --- | --- |
| `GITLAB_API_HOST` | `hosts.<hostname>.api_host`, or `hosts.<hostname>` if empty | Hostname found in the Git URL | Specify the host where the API endpoint is found. Useful when there are separate (sub)domains or hosts for Git and the API endpoint. |
| `GITLAB_CLIENT_ID` | `hosts.<hostname>.client_id` | Client-ID for GitLab.com. | A custom Client-ID generated by the GitLab OAuth 2.0 application. |
| `GITLAB_GROUP` | - | - | Default GitLab group used for listing merge requests, issues and variables. Only used if no `--group` option is given. |
| `GITLAB_HOST` | `host` (this is the default host `glab` will use when the current directory is not a `git` directory) | `https://gitlab.com` | Alias of `GITLAB_URI`. |
| `GITLAB_REPO` | - | - | Default GitLab repository used for commands accepting the `--repo` option. Only used if no `--repo` option is given. |
| `GITLAB_TOKEN` | `hosts.<hostname>.token` | - | an authentication token for API requests. Setting this avoids being prompted to authenticate and overrides any previously stored credentials. Can be set in the config with `glab config set token xxxxxx`. |
| `GITLAB_URI` | not applicable | not applicable | Alias of `GITLAB_HOST`. |

### `glab` configuration variables [Link to heading 'glab configuration variables'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#glab-configuration-variables)

| Token name | In `config.yml` | Default value if [not set](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration) | Description |
| --- | --- | --- | --- |
| `BROWSER` | `browser` | system default | The web browser to use for opening links. Can be set in the configuration with `glab config set browser mybrowser`. |
| `FORCE_HYPERLINKS` | `display_hyperlinks` | `false` | Set to `true` to force hyperlinks to be output, even when not outputting to a TTY. |
| `GITLAB_RELEASE_ASSETS_USE_PACKAGE_REGISTRY` | - | - | When `true` or `1`, the `glab release create` command uploads release assets to the generic package registry of the project. Can be overridden with the `--use-package-registry` flag. |
| `GLAB_CHECK_UPDATE` | - | - | Set to `true` to force an update check. |
| `GLAB_CONFIG_DIR` | - | `~/.config/glab-cli/` | Directory where the `glab` global configuration file is located. Can be set in the config with `glab config set remote_alias origin`. |
| `GLAB_DEBUG_HTTP` | - | `false` | Set to true to output HTTP transport information (request / response). |
| `GLAB_SEND_TELEMETRY` | `telemetry` | `true` | Set to `false` to prevent command usage data from being sent to your GitLab instance. |
| `GLAMOUR_STYLE` | `glamour_style` | `dark` | Environment variable to set your desired Markdown renderer style. Available options are (`dark`, `light`, `notty`) or set a [custom style](https://github.com/charmbracelet/glamour#styles). |
| `NO_COLOR` | - | `true` | Set to any value to avoid printing ANSI escape sequences for color output. |
| `NO_PROMPT` | `no_prompt` | `false` | Set to `true` to disable prompts. |
| `VISUAL`, `EDITOR` | `editor` | `nano` | (in order of precedence) The editor tool to use for authoring text. Can be set in the config with `glab config set editor vim`. |

### Other variables [Link to heading 'Other variables'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#other-variables)

| Token name | In `config.yml` | Default value if [not set](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md#configuration) | Description |
| --- | --- | --- | --- |
| `DEBUG` | `debug` | `false` | Set to `true` to output more information for each command, like Git commands, expanded aliases, and DNS error details. |
| `GIT_REMOTE_URL_VAR` | not applicable | not applicable | Alias of `REMOTE_ALIAS`. |
| `REMOTE_ALIAS` | `remote_alias` | - | `git remote` variable or alias that contains the GitLab URL. Alias: `GIT_REMOTE_URL_VAR` |

#### Variable deprecation [Link to heading 'Variable deprecation'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#variable-deprecation)

In `glab` version 2.0.0 and later, all `glab` environment variables are prefixed with `GLAB_`.
For more information about this deprecation, see [issue 7999](https://gitlab.com/gitlab-org/cli/-/issues/7999 "glab cli environment variable deprecation (GLAB_ prefix) feedback issue").

### Token and environment variable precedence [Link to heading 'Token and environment variable precedence'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#token-and-environment-variable-precedence)

GLab uses tokens in this order:

1. Environment variable (`GITLAB_TOKEN`).
2. Configuration file (`$HOME/.config/glab-cli/config.yml`).

### Debugging [Link to heading 'Debugging'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#debugging)

When the `DEBUG` environment variable is set to `true`, `glab` outputs more logging information, including:

- Underlying Git commands.
- Expanded aliases.
- DNS error details.

## Troubleshooting [Link to heading 'Troubleshooting'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#troubleshooting)

For troubleshooting information, see the
[GitLab documentation for the CLI](https://docs.gitlab.com/editor_extensions/gitlab_cli/#troubleshooting).

## Issues [Link to heading 'Issues'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#issues)

If you have an issue: report it on the [issue tracker](https://gitlab.com/gitlab-org/cli/-/issues)

## Contributing [Link to heading 'Contributing'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#contributing)

Feel like contributing? That's awesome! We have a [contributing guide](https://gitlab.com/gitlab-org/cli/-/blob/main/CONTRIBUTING.md) and [Code of conduct](https://gitlab.com/gitlab-org/cli/-/blob/main/CODE_OF_CONDUCT.md) to help guide you.

### Versioning [Link to heading 'Versioning'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#versioning)

This project follows the [SemVer](https://github.com/semver/semver) specification.

### Classify version changes [Link to heading 'Classify version changes'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#classify-version-changes)

- If deleting a command, changing how it behaves, or adding a new **required** flag, the release must use a new `MAJOR` revision.
- If adding a new command or **optional** flag, the release must use a new `MINOR` revision.
- If fixing a bug, the release must use a new `PATCH` revision.

### Compatibility [Link to heading 'Compatibility'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#compatibility)

We do our best to introduce breaking changes only when releasing a new `MAJOR` version.
Unfortunately, there are situations where this is not possible, and we may introduce
a breaking change in a `MINOR` or `PATCH` version. Some of situations where we may do so:

- If a security issue is discovered, and the solution requires a breaking change,
we may introduce such a change to resolve the issue and protect our users.
- If a feature was not working as intended, and the bug fix requires a breaking change,
the bug fix may be introduced to ensure the functionality works as intended.
- When feature behavior is overwhelmingly confusing due to a vague specification
on how it should work. In such cases, we may refine the specification
to remove the ambiguity, and introduce a breaking change that aligns with the
refined specification. For an example of this, see
[merge request 1382](https://gitlab.com/gitlab-org/cli/-/merge_requests/1382#note_1686888887 "fix: prefer CTRL+D for cancel job (#1286)").
- Experimental features are not guaranteed to be stable, and can be modified or
removed without a breaking change.

Breaking changes are a last resort, and we try our best to only introduce them when absolutely necessary.

## Inspiration [Link to heading 'Inspiration'](https://gitlab.com/gitlab-org/cli/-/blob/main/README.md\#inspiration)

The GitLab CLI was adopted from [Clement Sam](https://gitlab.com/profclems) in 2022 to serve as the official CLI of GitLab. Over the years the project has been inspired by both the [GitHub CLI](https://github.com/cli/cli) and [Zaq? Wiedmann's](https://gitlab.com/zaquestion) [lab](https://github.com/zaquestion/lab).

Lab has served as the foundation for many of the GitLab CI/CD commands including `ci view` and `ci trace`.