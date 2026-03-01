# Subtask task-20.8: Create Discord and Email Configuration Schemas

## Parent Task
Task 20

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Implement Effect Schema definitions for DiscordConfig and EmailConfig with platform-specific validation rules and required field constraints.

## Dependencies
None

## Implementation Details
Define DiscordConfig schema with webhook URL, guild ID, channel ID, and bot token fields. Create EmailConfig schema with SMTP settings, recipient lists, template options, and authentication credentials. Include validation for Discord snowflake IDs, email address formats, and SMTP configuration requirements. Use branded types for secure credential handling.

## Test Strategy
Validation tests for Discord and email configuration edge cases

---
*Project: alerthub*
