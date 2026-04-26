# SOC 2 Security Checklist: AWS & GitHub

This document provides a comprehensive checklist of least privilege and zero trust security controls for achieving and maintaining SOC 2 compliance. Controls are organized by domain and mapped to SOC 2 Trust Service Criteria (TSC).

---

## Table of Contents

- [AWS Identity & Access Management (IAM)](#aws-identity--access-management-iam)
- [AWS Network Security](#aws-network-security)
- [AWS Data Protection](#aws-data-protection)
- [AWS Logging & Monitoring](#aws-logging--monitoring)
- [AWS Infrastructure Security](#aws-infrastructure-security)
- [AWS Incident Response](#aws-incident-response)
- [GitHub Access Control](#github-access-control)
- [GitHub Repository Security](#github-repository-security)
- [GitHub CI/CD Security](#github-cicd-security)
- [GitHub Secret Management](#github-secret-management)
- [GitHub Audit & Compliance](#github-audit--compliance)
- [SOC 2 Trust Service Criteria Mapping](#soc-2-trust-service-criteria-mapping)
- [Documentation Requirements](#documentation-requirements)
- [Periodic Review Schedule](#periodic-review-schedule)

---

## AWS Identity & Access Management (IAM)

### Credential Management

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Eliminate root account usage | Root account should only be used for initial setup and break-glass scenarios | CC6.1 | Critical |
| Enable MFA on root account | Hardware MFA token preferred; store recovery codes securely offline | CC6.1 | Critical |
| No long-lived access keys | Use IAM roles with temporary credentials; delete all static access keys | CC6.1, CC6.2 | Critical |
| Use OIDC federation for CI/CD | GitHub Actions, GitLab CI should use OIDC, not stored AWS credentials | CC6.1 | Critical |
| Enforce MFA for all IAM users | Deny API access without MFA via IAM policy conditions | CC6.1 | High |
| Password policy enforcement | Minimum 14 characters, complexity requirements, 90-day rotation | CC6.1 | High |
| Credential rotation automation | Automate rotation of any remaining service account credentials | CC6.2 | High |

### Least Privilege Access

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use IAM roles over users | Prefer roles for applications, services, and cross-account access | CC6.1 | Critical |
| Scope policies to specific resources | Avoid `Resource: "*"` - specify exact ARNs where possible | CC6.1 | Critical |
| Deny by default | Start with zero permissions, add only what's needed | CC6.1 | Critical |
| Use AWS managed policies sparingly | Custom policies scoped to actual requirements preferred | CC6.1 | High |
| Implement permission boundaries | Prevent privilege escalation even if policies are misconfigured | CC6.1 | High |
| Separate read/write permissions | Read-only roles for monitoring, write access only where needed | CC6.1 | High |
| Time-bound access for sensitive operations | Use AWS SSO with session limits; consider just-in-time access tools | CC6.2 | Medium |

### Access Analysis & Review

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable IAM Access Analyzer | Continuously analyze policies for overly permissive access | CC6.1 | Critical |
| Review IAM credential report | Monthly review for unused credentials, access key age | CC6.2 | High |
| Use Access Advisor | Identify unused permissions for policy refinement | CC6.1 | High |
| Audit cross-account access | Document and review all cross-account IAM roles | CC6.1 | High |
| Remove unused IAM users/roles | Delete accounts not used in 90+ days | CC6.2 | High |

### AWS Organizations & Multi-Account

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Separate accounts by environment | Dev, staging, production in isolated AWS accounts | CC6.1 | Critical |
| Dedicated security/audit account | Centralized logging, security tools in separate account | CC6.1, CC7.2 | Critical |
| Implement SCPs | Organization-wide guardrails that cannot be overridden | CC6.1 | Critical |
| Deny disabling CloudTrail via SCP | Prevent any account from turning off audit logging | CC7.2 | Critical |
| Deny public S3 via SCP | Organization-wide prevention of public bucket creation | CC6.1 | Critical |
| Restrict region usage via SCP | Limit to approved regions only | CC6.1 | High |
| Deny root account actions via SCP | Prevent root usage except in management account | CC6.1 | High |

### AWS SSO / Identity Center

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use AWS SSO for human access | Centralized identity management with IdP integration | CC6.1 | Critical |
| Integrate with corporate IdP | SAML/OIDC federation with Okta, Azure AD, etc. | CC6.1 | Critical |
| Enforce MFA at IdP level | MFA required before AWS access is granted | CC6.1 | Critical |
| Session duration limits | Maximum 4-8 hour sessions for console access | CC6.2 | High |
| Permission sets per role | Developer, Admin, ReadOnly with appropriate scoping | CC6.1 | High |
| Just-in-time access for elevated privileges | Temporary admin access with approval workflow | CC6.2 | Medium |

---

## AWS Network Security

### VPC Architecture

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use private subnets for databases | No direct internet access for data stores | CC6.6 | Critical |
| Use private subnets for internal services | Application servers in private subnets behind load balancers | CC6.6 | Critical |
| NAT Gateway for outbound traffic | Controlled egress from private subnets | CC6.6 | High |
| VPC per environment | Isolated VPCs for dev, staging, production | CC6.6 | High |
| VPC peering with least privilege | Only required routes between VPCs | CC6.6 | High |
| Transit Gateway for complex topologies | Centralized network management with route controls | CC6.6 | Medium |

### Security Groups

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Default deny all inbound | Security groups start with no inbound rules | CC6.6 | Critical |
| Explicit allow by port and source | Only required ports from specific CIDR/security groups | CC6.6 | Critical |
| No 0.0.0.0/0 for SSH/RDP | Management access via bastion or SSM only | CC6.6 | Critical |
| Security group references over CIDR | Use security group IDs for internal communication | CC6.6 | High |
| Separate security groups by function | Web, app, database tiers have distinct groups | CC6.6 | High |
| Document security group purposes | Clear naming and descriptions for audit | CC6.6 | Medium |

### Network ACLs

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use NACLs as secondary defense | Defense in depth with subnet-level controls | CC6.6 | High |
| Deny known malicious IP ranges | Block traffic from threat intelligence feeds | CC6.6 | Medium |
| Restrict ephemeral port ranges | Limit return traffic ports where feasible | CC6.6 | Low |

### Connectivity & Access

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use AWS Systems Manager Session Manager | No SSH keys, no bastion hosts, audited access | CC6.1, CC7.2 | Critical |
| VPC endpoints for AWS services | Keep traffic off public internet | CC6.6 | High |
| PrivateLink for third-party services | Private connectivity to SaaS providers | CC6.6 | High |
| AWS Client VPN or Direct Connect | Secure access from corporate networks | CC6.6 | High |
| Disable public IPs by default | Use NAT/load balancers for internet access | CC6.6 | High |

### DDoS Protection

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable AWS Shield Standard | Automatic protection for L3/L4 attacks | CC6.6 | Critical |
| Consider Shield Advanced | Enhanced DDoS protection with 24/7 response team | CC6.6 | Medium |
| CloudFront for public endpoints | Edge caching reduces origin exposure | CC6.6 | High |
| WAF for application layer | Protect against OWASP Top 10 attacks | CC6.6 | High |

---

## AWS Data Protection

### Encryption at Rest

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| S3 default encryption | SSE-S3 minimum, SSE-KMS for sensitive data | CC6.7 | Critical |
| RDS encryption enabled | Enable at creation (cannot be added later) | CC6.7 | Critical |
| EBS volume encryption | Default encryption for all new volumes | CC6.7 | Critical |
| DynamoDB encryption | AWS owned or customer managed KMS key | CC6.7 | Critical |
| ElastiCache encryption | At-rest and in-transit encryption | CC6.7 | Critical |
| Secrets Manager encryption | KMS encryption for all secrets | CC6.7 | Critical |
| Backup encryption | Ensure all backups are encrypted | CC6.7 | High |
| EFS encryption | Enable for shared file systems | CC6.7 | High |

### Encryption in Transit

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enforce TLS 1.2+ | Disable TLS 1.0/1.1 on all endpoints | CC6.7 | Critical |
| S3 bucket policy requiring HTTPS | Deny requests without `aws:SecureTransport` | CC6.7 | Critical |
| RDS SSL/TLS connections | Enforce encrypted database connections | CC6.7 | Critical |
| Load balancer TLS termination | HTTPS listeners with modern cipher suites | CC6.7 | Critical |
| ElastiCache in-transit encryption | Enable for Redis/Memcached | CC6.7 | High |
| Internal service mesh TLS | mTLS between microservices | CC6.7 | High |

### KMS Key Management

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Customer managed keys (CMK) for sensitive data | Control key policies and rotation | CC6.7 | Critical |
| Enable automatic key rotation | Annual rotation for symmetric keys | CC6.7 | High |
| Least privilege key policies | Only authorized services/users can use keys | CC6.7 | High |
| Separate keys by environment | Production uses different keys than dev | CC6.7 | High |
| Audit key usage via CloudTrail | Monitor encrypt/decrypt operations | CC7.2 | High |
| Key deletion protection | Multi-day waiting period, admin approval | CC6.7 | Medium |

### S3 Security

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Block Public Access (account level) | Enable all four BPA settings at account level | CC6.1 | Critical |
| Block Public Access (bucket level) | Defense in depth at bucket level | CC6.1 | Critical |
| Bucket policies with least privilege | Explicit principals, no `Principal: "*"` | CC6.1 | Critical |
| Enable versioning | Protect against accidental deletion | CC6.7 | High |
| Enable MFA Delete | Require MFA to delete versions | CC6.7 | High |
| Object Lock for compliance data | WORM protection for regulatory requirements | CC6.7 | Medium |
| Access Points for shared buckets | Scoped access for different consumers | CC6.1 | Medium |
| S3 Inventory and Storage Lens | Visibility into bucket contents and access patterns | CC7.2 | Medium |

### Secrets Management

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use AWS Secrets Manager | Centralized secrets with automatic rotation | CC6.7 | Critical |
| Never hardcode secrets | No credentials in code, configs, or environment variables | CC6.7 | Critical |
| Automatic rotation enabled | Database credentials, API keys rotated automatically | CC6.7 | High |
| Least privilege secret access | IAM policies scoped to specific secrets | CC6.1 | High |
| Audit secret access | CloudTrail logging of GetSecretValue calls | CC7.2 | High |
| Use Parameter Store for config | Non-sensitive configuration in SSM Parameter Store | CC6.7 | Medium |

---

## AWS Logging & Monitoring

### CloudTrail

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable in all regions | Trail covers all regions, including future regions | CC7.2 | Critical |
| Enable for all accounts | Organization trail for centralized logging | CC7.2 | Critical |
| Log file validation | Detect tampering with log files | CC7.2 | Critical |
| S3 bucket for long-term storage | Separate audit account with restricted access | CC7.2 | Critical |
| Enable data events for S3 | Track object-level operations on sensitive buckets | CC7.2 | High |
| Enable data events for Lambda | Track invocations of sensitive functions | CC7.2 | High |
| CloudWatch Logs integration | Real-time log analysis and alerting | CC7.2 | High |
| Athena for log analysis | Query logs for investigations | CC7.2 | Medium |

### CloudWatch

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Alarms for security events | Root login, IAM changes, security group changes | CC7.2, CC7.3 | Critical |
| Log retention policies | Minimum 1 year retention for compliance | CC7.2 | Critical |
| Cross-account log aggregation | Central logging account for all CloudWatch logs | CC7.2 | High |
| Custom metrics for application security | Failed logins, authorization failures | CC7.2 | High |
| Log Insights queries saved | Pre-built queries for common investigations | CC7.2 | Medium |
| Anomaly detection | ML-based alerting on unusual patterns | CC7.2 | Medium |

### VPC Flow Logs

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable for all VPCs | Capture ACCEPT and REJECT traffic | CC7.2 | Critical |
| S3 destination for long-term storage | Cost-effective retention | CC7.2 | High |
| CloudWatch for real-time analysis | Alert on rejected traffic patterns | CC7.2 | High |
| Include metadata fields | Source/destination IPs, ports, protocols | CC7.2 | High |

### AWS Config

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable in all regions | Track resource configuration changes | CC7.2 | Critical |
| Conformance packs for compliance | Pre-built rule sets for SOC 2, CIS, etc. | CC7.2 | High |
| Custom rules for organization policies | Enforce specific requirements | CC7.2 | High |
| Remediation actions | Auto-fix non-compliant resources | CC7.2 | Medium |
| Aggregator for multi-account | Centralized compliance dashboard | CC7.2 | High |

### Security Hub

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable Security Hub | Centralized security findings | CC7.2 | Critical |
| Enable CIS AWS Foundations benchmark | Automated compliance checking | CC7.2 | High |
| Enable AWS Foundational Security Best Practices | Additional security checks | CC7.2 | High |
| Integrate GuardDuty, Inspector, IAM Access Analyzer | Aggregated findings | CC7.2 | High |
| Custom insights for priority findings | Focus on critical issues | CC7.2 | Medium |

---

## AWS Infrastructure Security

### Compute Security

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use IMDSv2 only | Disable IMDSv1 to prevent SSRF attacks | CC6.6 | Critical |
| EC2 instance profiles over access keys | Roles for EC2 instances, not credentials | CC6.1 | Critical |
| AMI hardening | CIS benchmarks, remove unnecessary packages | CC6.8 | High |
| Systems Manager Patch Manager | Automated patching for EC2 instances | CC6.8 | High |
| Inspector for vulnerability scanning | Continuous scanning of EC2, ECR, Lambda | CC6.8 | High |
| No public EC2 instances in production | All access via load balancers or private endpoints | CC6.6 | High |

### Container Security

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| ECR image scanning | Scan on push, block vulnerable images | CC6.8 | Critical |
| Minimal base images | Distroless or Alpine, no unnecessary packages | CC6.8 | High |
| No privileged containers | Drop all capabilities, read-only root filesystem | CC6.8 | High |
| ECS task IAM roles | Least privilege per task definition | CC6.1 | High |
| EKS IRSA (IAM Roles for Service Accounts) | Pod-level IAM permissions | CC6.1 | High |
| Private ECR repositories | No public container registries | CC6.6 | High |
| Image signing and verification | Ensure image integrity | CC6.8 | Medium |

### Serverless Security

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Least privilege Lambda execution roles | Only required permissions per function | CC6.1 | Critical |
| Lambda in VPC for data access | Functions accessing databases in private subnets | CC6.6 | High |
| Reserved concurrency limits | Prevent runaway costs and DoS | CC6.6 | High |
| Environment variable encryption | KMS encryption for sensitive config | CC6.7 | High |
| Dead letter queues | Capture and analyze failed invocations | CC7.2 | Medium |

### Database Security

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| No public database endpoints | RDS, ElastiCache, etc. in private subnets only | CC6.6 | Critical |
| Enable deletion protection | Prevent accidental database deletion | CC6.7 | Critical |
| IAM database authentication | Where supported (Aurora, RDS PostgreSQL/MySQL) | CC6.1 | High |
| Automated backups enabled | Point-in-time recovery capability | CC6.7 | High |
| Cross-region backups for DR | Replicate to secondary region | CC6.7 | High |
| Parameter groups hardened | Disable unnecessary features, enforce SSL | CC6.8 | High |
| Audit logging enabled | Database activity streams, general/slow query logs | CC7.2 | High |

---

## AWS Incident Response

### Threat Detection

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable GuardDuty | Intelligent threat detection | CC7.2, CC7.3 | Critical |
| GuardDuty in all accounts/regions | Organization-wide coverage | CC7.2 | Critical |
| Integrate with alerting system | PagerDuty, Slack, SNS for notifications | CC7.3 | Critical |
| Malware Protection for S3 | Scan uploaded files | CC7.2 | High |
| EKS/ECS runtime monitoring | Container threat detection | CC7.2 | High |
| Suppress known false positives carefully | Document suppression rules | CC7.2 | Medium |

### Forensics Readiness

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| CloudTrail logs preserved | Immutable, long-term retention | CC7.2 | Critical |
| EBS snapshot capability | Ability to snapshot compromised instances | CC7.3 | High |
| Memory acquisition tools ready | Prepared AMIs with forensic tools | CC7.3 | Medium |
| Isolated forensics VPC | Quarantine area for analysis | CC7.3 | Medium |
| Playbooks documented | Step-by-step incident response procedures | CC7.3 | High |

### Automated Response

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Auto-remediation for critical findings | Lambda functions to isolate compromised resources | CC7.3 | High |
| Revoke compromised credentials | Automated response to credential exposure | CC7.3 | High |
| Quarantine compromised instances | Remove from security groups, isolate | CC7.3 | High |
| Notify security team | Immediate alerting on high-severity findings | CC7.3 | Critical |

---

## GitHub Access Control

### Organization Settings

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enforce SAML SSO | Require IdP authentication for organization access | CC6.1 | Critical |
| Require 2FA for all members | Block users without 2FA | CC6.1 | Critical |
| Disable forking to personal accounts | Prevent code exfiltration | CC6.1 | High |
| Restrict repository creation | Only admins or specific teams can create repos | CC6.1 | High |
| Restrict repository visibility changes | Prevent accidental public exposure | CC6.1 | Critical |
| Disable OAuth app access | Or require admin approval | CC6.1 | High |
| Restrict GitHub App installations | Admin approval required | CC6.1 | High |
| IP allow lists | Restrict access to corporate network/VPN | CC6.1 | Medium |

### Team-Based Access

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use Teams over individual permissions | Group-based access management | CC6.1 | Critical |
| Separate teams by function | Engineering, Security, Ops with different access | CC6.1 | High |
| No direct collaborator additions | All access via team membership | CC6.1 | High |
| Regular team membership reviews | Quarterly access reviews | CC6.2 | High |
| Nested teams for hierarchy | Parent teams inherit permissions appropriately | CC6.1 | Medium |

### Repository Permissions

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Default to minimum permission | Read access by default, write only when needed | CC6.1 | Critical |
| Admin access tightly controlled | Only repo maintainers and security team | CC6.1 | Critical |
| Maintain permission for most developers | Push access without settings changes | CC6.1 | High |
| Read access for monitoring/audit tools | Service accounts with minimal permissions | CC6.1 | High |

### Outside Collaborators

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Require admin approval | External collaborators must be approved | CC6.1 | Critical |
| Time-limited access | Review and revoke contractor access regularly | CC6.2 | High |
| Limit to specific repositories | No organization-wide access for externals | CC6.1 | High |
| Audit collaborator list regularly | Monthly review of external access | CC6.2 | High |

---

## GitHub Repository Security

### Branch Protection

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Protect main/production branches | Prevent direct pushes | CC8.1 | Critical |
| Require pull request reviews | Minimum 2 reviewers for production code | CC8.1 | Critical |
| Require status checks to pass | CI must succeed before merge | CC8.1 | Critical |
| Require conversation resolution | All review comments addressed | CC8.1 | High |
| Require signed commits | Verify commit author identity | CC6.1 | High |
| Require linear history | No merge commits, cleaner history | CC8.1 | Medium |
| Restrict who can push | Only specific teams can merge | CC6.1 | High |
| Disable force pushes | Prevent history rewriting | CC8.1 | Critical |
| Disable branch deletion | Protect important branches | CC8.1 | High |
| Require deployments to succeed | Integration with deployment status | CC8.1 | Medium |

### CODEOWNERS

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| CODEOWNERS file present | Define required reviewers per path | CC8.1 | Critical |
| Security team owns sensitive paths | `.github/`, `infra/`, `secrets/`, security configs | CC8.1 | Critical |
| Require CODEOWNERS review | Enable in branch protection settings | CC8.1 | Critical |
| Keep CODEOWNERS current | Update when team structure changes | CC8.1 | High |

### Code Scanning

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable CodeQL analysis | GitHub's SAST for security vulnerabilities | CC6.8 | Critical |
| Scan on every PR | Block merge on high/critical findings | CC6.8 | Critical |
| Scan on push to default branch | Continuous monitoring | CC6.8 | High |
| Custom CodeQL queries | Organization-specific security rules | CC6.8 | Medium |
| Third-party SAST integration | Semgrep, SonarQube for additional coverage | CC6.8 | Medium |

### Dependency Management

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable Dependabot alerts | Automatic vulnerability notifications | CC6.8 | Critical |
| Enable Dependabot security updates | Auto-create PRs for vulnerable dependencies | CC6.8 | Critical |
| Enable Dependabot version updates | Keep dependencies current | CC6.8 | High |
| Review and merge Dependabot PRs promptly | SLA for security updates (e.g., 7 days) | CC6.8 | High |
| Lock file present | package-lock.json, Cargo.lock, etc. | CC6.8 | High |
| Audit dependencies before adding | Review new dependencies for security | CC6.8 | Medium |

---

## GitHub CI/CD Security

### Workflow Security

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Minimal workflow permissions | Use `permissions:` block with least privilege | CC6.1 | Critical |
| No `permissions: write-all` | Explicitly scope each permission | CC6.1 | Critical |
| Pin actions to SHA | Use commit SHA, not tags (e.g., `@abc123` not `@v4`) | CC6.8 | Critical |
| Review third-party actions | Audit actions before use, prefer official actions | CC6.8 | High |
| Fork PR restrictions | Require approval for first-time contributors | CC6.1 | High |
| No secrets in workflow logs | Use `::add-mask::` for sensitive outputs | CC6.7 | High |
| Workflow path restrictions | Limit who can modify `.github/workflows/` | CC8.1 | Critical |

### Environment Protection

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use environments for deployments | Separate dev, staging, production | CC8.1 | Critical |
| Required reviewers for production | Human approval before production deploy | CC8.1 | Critical |
| Wait timer | Delay production deployments for review window | CC8.1 | High |
| Branch restrictions | Only main branch can deploy to production | CC8.1 | High |
| Environment-specific secrets | Different credentials per environment | CC6.7 | Critical |

### OIDC Authentication

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use OIDC for cloud providers | No stored cloud credentials in secrets | CC6.1 | Critical |
| Scope OIDC to specific repos | Trust policy limits which repos can assume roles | CC6.1 | Critical |
| Scope OIDC to specific branches | Production role only from main branch | CC6.1 | High |
| Scope OIDC to specific environments | Match environment to cloud role | CC6.1 | High |
| Short session duration | Minimal token lifetime (1 hour max) | CC6.2 | High |

### Self-Hosted Runners

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Prefer GitHub-hosted runners | Ephemeral, managed security | CC6.8 | High |
| Isolate self-hosted runners | Dedicated VMs, not shared infrastructure | CC6.6 | Critical |
| Auto-scaling runner groups | Ephemeral runners that clean up after each job | CC6.8 | High |
| Runner groups per repository | Limit which repos can use which runners | CC6.1 | High |
| Regular runner updates | Keep runner software current | CC6.8 | High |
| Network isolation | Runners in private network with controlled egress | CC6.6 | High |

---

## GitHub Secret Management

### GitHub Secrets

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Use GitHub Secrets for sensitive values | Never hardcode in workflows or code | CC6.7 | Critical |
| Environment secrets over repo secrets | Scope secrets to deployment environments | CC6.7 | High |
| Organization secrets with access control | Limit which repos can access org secrets | CC6.7 | High |
| Rotate secrets regularly | Scheduled rotation for API keys, tokens | CC6.7 | High |
| Audit secret access | Review Actions logs for secret usage | CC7.2 | Medium |

### Secret Scanning

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable secret scanning | Automatic detection of leaked secrets | CC6.7 | Critical |
| Enable push protection | Block pushes containing secrets | CC6.7 | Critical |
| Custom patterns | Add organization-specific secret patterns | CC6.7 | High |
| Immediate response to alerts | Rotate exposed secrets immediately | CC7.3 | Critical |
| Review validity of detected secrets | Confirm if secrets are real and active | CC6.7 | High |

### Code Hygiene

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Comprehensive .gitignore | Exclude credentials, keys, env files | CC6.7 | Critical |
| Pre-commit hooks | Local secret detection before push | CC6.7 | High |
| No .env files in repository | Use .env.example with placeholders | CC6.7 | Critical |
| Audit git history | Ensure no secrets in historical commits | CC6.7 | High |
| BFG or git-filter-repo for cleanup | Remove secrets from history if found | CC6.7 | High |

---

## GitHub Audit & Compliance

### Audit Logging

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Enable audit log streaming | Stream to SIEM (Splunk, Datadog, etc.) | CC7.2 | Critical |
| Retain audit logs 1+ year | Meet compliance retention requirements | CC7.2 | Critical |
| Monitor for suspicious activity | Alert on permission changes, repository visibility changes | CC7.2 | High |
| Review audit logs regularly | Weekly review of significant events | CC7.2 | High |
| API audit logging | Track programmatic access | CC7.2 | High |

### Access Reviews

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Quarterly access reviews | Review all user and team permissions | CC6.2 | Critical |
| Review outside collaborators monthly | External access requires ongoing justification | CC6.2 | High |
| Review OAuth/GitHub App access | Remove unused integrations | CC6.2 | High |
| Document access decisions | Record approval/denial of access requests | CC6.2 | High |
| Automated access reporting | Generate reports for compliance audits | CC6.2 | Medium |

### Compliance Features

| Control | Description | SOC 2 Criteria | Priority |
|---------|-------------|----------------|----------|
| Repository archive policy | Archive inactive repositories | CC6.2 | Medium |
| Enterprise Managed Users (EMU) | Full identity control via IdP (Enterprise only) | CC6.1 | High |
| Legal hold capability | Preserve data for legal requirements | CC7.2 | Medium |
| Export functionality | Ability to export repos for audit | CC7.2 | Medium |

---

## SOC 2 Trust Service Criteria Mapping

### Security (CC6.x) - Protection of Information and Systems

| Criteria | Description | Key Controls |
|----------|-------------|--------------|
| CC6.1 | Logical access security | IAM, MFA, SSO, least privilege, access reviews |
| CC6.2 | Access provisioning/deprovisioning | Onboarding/offboarding, access reviews, credential rotation |
| CC6.3 | Access removal | Timely deprovisioning, automated removal |
| CC6.6 | Network security | VPCs, security groups, private subnets, WAF |
| CC6.7 | Data security | Encryption at rest/transit, KMS, secrets management |
| CC6.8 | System security | Patching, vulnerability scanning, hardening |

### Availability (A1.x) - System Availability

| Criteria | Description | Key Controls |
|----------|-------------|--------------|
| A1.1 | Availability commitments | SLAs, redundancy, multi-AZ |
| A1.2 | Backup and recovery | Automated backups, cross-region replication, DR testing |
| A1.3 | Recovery testing | Regular DR drills, documented recovery procedures |

### Processing Integrity (PI1.x) - Accurate and Complete Processing

| Criteria | Description | Key Controls |
|----------|-------------|--------------|
| PI1.1 | Processing integrity | Input validation, CI/CD gates, automated testing |
| PI1.2 | Error handling | Logging, monitoring, alerting |
| PI1.3 | Data quality | Validation rules, data integrity checks |

### Confidentiality (C1.x) - Protection of Confidential Information

| Criteria | Description | Key Controls |
|----------|-------------|--------------|
| C1.1 | Confidential information | Data classification, encryption, access controls |
| C1.2 | Disposal | Secure deletion, data retention policies |

### Common Criteria (CC7.x - CC9.x) - Operations and Risk Management

| Criteria | Description | Key Controls |
|----------|-------------|--------------|
| CC7.1 | Configuration management | IaC, change tracking, version control |
| CC7.2 | Monitoring | CloudTrail, CloudWatch, GitHub audit logs, alerting |
| CC7.3 | Incident response | GuardDuty, playbooks, automated response |
| CC7.4 | Change management | CI/CD, PR reviews, testing, approvals |
| CC8.1 | Change management | Branch protection, code review, deployment gates |
| CC9.1 | Risk management | Vulnerability scanning, threat modeling |
| CC9.2 | Vendor management | Third-party risk assessment, SLA monitoring |

---

## Documentation Requirements

### Required Policies

| Document | Description | Review Frequency |
|----------|-------------|-----------------|
| Information Security Policy | Overall security governance | Annual |
| Access Control Policy | Who can access what and approval process | Annual |
| Data Classification Policy | How data is categorized and protected | Annual |
| Encryption Policy | Encryption standards and requirements | Annual |
| Password Policy | Password requirements and rotation | Annual |
| Incident Response Plan | Steps for security incident handling | Annual |
| Business Continuity Plan | DR procedures and RTO/RPO targets | Annual |
| Change Management Policy | How changes are reviewed and deployed | Annual |
| Vendor Management Policy | Third-party risk assessment process | Annual |
| Data Retention Policy | How long data is kept and when deleted | Annual |

### Required Procedures

| Document | Description | Review Frequency |
|----------|-------------|-----------------|
| User Access Review Procedure | How access reviews are conducted | Quarterly |
| Onboarding/Offboarding Checklist | Steps for provisioning/deprovisioning | Per event |
| Incident Response Playbooks | Specific runbooks for incident types | Semi-annual |
| Backup Verification Procedure | How backups are tested | Monthly |
| Vulnerability Management Procedure | How vulnerabilities are triaged and remediated | Ongoing |
| Penetration Testing Procedure | Scope and frequency of pen tests | Annual |

### Evidence Artifacts

| Artifact | Description | Collection Frequency |
|----------|-------------|---------------------|
| Access review records | Screenshots/exports of quarterly reviews | Quarterly |
| Vulnerability scan reports | Inspector, CodeQL, Dependabot reports | Continuous |
| Penetration test reports | Third-party or internal pen test results | Annual |
| Incident tickets | Records of security incidents and response | Per incident |
| Change records | PR history, deployment logs | Continuous |
| Training records | Security awareness training completion | Annual |
| Audit logs | CloudTrail, GitHub audit log exports | Continuous |

---

## Periodic Review Schedule

### Daily

- [ ] Review GuardDuty findings
- [ ] Review secret scanning alerts
- [ ] Review Dependabot security alerts
- [ ] Monitor CloudWatch alarms

### Weekly

- [ ] Review GitHub audit log for suspicious activity
- [ ] Review CloudTrail events for anomalies
- [ ] Triage open vulnerability findings
- [ ] Review failed deployment logs

### Monthly

- [ ] Review outside collaborator list
- [ ] Review OAuth/GitHub App access
- [ ] Verify backup restoration capability
- [ ] Review IAM credential report
- [ ] Update vulnerability remediation progress

### Quarterly

- [ ] Conduct user access review (IAM + GitHub)
- [ ] Review and update CODEOWNERS
- [ ] Review security group rules
- [ ] Review S3 bucket policies
- [ ] Test incident response procedures
- [ ] Update risk register

### Annually

- [ ] Review and update all security policies
- [ ] Conduct penetration test
- [ ] Review SCPs and permission boundaries
- [ ] Conduct disaster recovery test
- [ ] Security awareness training
- [ ] Third-party vendor assessment
- [ ] SOC 2 audit preparation

---

## Quick Reference Commands

### AWS Security Checks

```bash
# Check for IAM users with console access but no MFA
aws iam generate-credential-report
aws iam get-credential-report --output text --query Content | base64 -d | grep -E "^[^,]+,.*,true,.*,false"

# List IAM users with access keys older than 90 days
aws iam list-users --query 'Users[*].UserName' --output text | \
  xargs -I {} aws iam list-access-keys --user-name {} --query 'AccessKeyMetadata[?CreateDate<`2024-01-01`]'

# Check S3 public access block status
aws s3control get-public-access-block --account-id $(aws sts get-caller-identity --query Account --output text)

# List security groups with 0.0.0.0/0 ingress
aws ec2 describe-security-groups --query 'SecurityGroups[?IpPermissions[?IpRanges[?CidrIp==`0.0.0.0/0`]]]'

# Check CloudTrail status
aws cloudtrail describe-trails --query 'trailList[*].[Name,IsMultiRegionTrail,LogFileValidationEnabled]'

# Check GuardDuty status
aws guardduty list-detectors

# List unencrypted EBS volumes
aws ec2 describe-volumes --query 'Volumes[?!Encrypted].[VolumeId,State]'

# List public RDS instances
aws rds describe-db-instances --query 'DBInstances[?PubliclyAccessible==`true`].[DBInstanceIdentifier]'
```

### GitHub Security Checks

```bash
# Check organization settings (requires gh CLI)
gh api orgs/{org} --jq '{two_factor_requirement_enabled, default_repository_permission}'

# List repositories without branch protection on main
gh api orgs/{org}/repos --paginate --jq '.[].name' | while read repo; do
  gh api repos/{org}/$repo/branches/main/protection 2>/dev/null || echo "$repo: No protection"
done

# Check for Dependabot alerts
gh api repos/{org}/{repo}/vulnerability-alerts

# List outside collaborators
gh api orgs/{org}/outside_collaborators --jq '.[].login'

# Check secret scanning status
gh api repos/{org}/{repo} --jq '{secret_scanning, secret_scanning_push_protection}'

# Audit organization members
gh api orgs/{org}/members --jq '.[].login'
```

---

## Additional Resources

- [AWS Well-Architected Security Pillar](https://docs.aws.amazon.com/wellarchitected/latest/security-pillar/welcome.html)
- [CIS AWS Foundations Benchmark](https://www.cisecurity.org/benchmark/amazon_web_services)
- [GitHub Security Best Practices](https://docs.github.com/en/code-security/getting-started/github-security-features)
- [SOC 2 Trust Service Criteria](https://www.aicpa.org/resources/article/soc-2-trust-service-criteria)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
