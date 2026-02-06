# Subtask 41.1: Setup Expo Notifications and configure FCM/APNs

## Parent Task
Task 41

## Subagent Type
implementer

## Agent
expo-deployer

## Parallelizable
Yes - can run concurrently

## Description
Install and configure Expo Notifications library, setup Firebase Cloud Messaging (FCM) for Android and Apple Push Notification service (APNs) for iOS with proper certificates and configuration files

## Dependencies
None

## Implementation Details
Install @expo/vector-icons and expo-notifications packages. Create Firebase project and download google-services.json for Android. Setup APNs certificates in Apple Developer Console and configure app.json/app.config.js with proper push notification settings. Configure notification channels for Android and notification categories for iOS.

## Test Strategy
See parent task acceptance criteria.
