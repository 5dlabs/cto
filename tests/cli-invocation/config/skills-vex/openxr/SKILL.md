# OpenXR Cross-Platform Development

## Overview

OpenXR is the Khronos Group's open standard for XR devices, providing a unified API across VR/AR headsets. Unity's OpenXR package implements this standard.

## Core Concepts

### OpenXR Architecture

```
Application Layer (Your Unity Code)
         ↓
Unity XR Plugin System
         ↓
OpenXR Loader (com.unity.xr.openxr)
         ↓
OpenXR Runtime (Oculus Runtime, SteamVR, etc.)
         ↓
Hardware (Quest, Index, Vive, etc.)
```

### Unity OpenXR Setup

1. **Install Package**: `com.unity.xr.openxr` via Package Manager
2. **Enable Plugin**: Project Settings → XR Plug-in Management → OpenXR
3. **Add Features**: Enable interaction profiles and extensions
4. **Configure Loader**: Set as active XR loader

## Interaction Profiles

OpenXR uses interaction profiles to abstract input across devices:

| Profile | Devices |
|---------|---------|
| `Oculus Touch Controller` | Quest, Rift |
| `Valve Index Controller` | Index |
| `HTC Vive Controller` | Vive, Vive Pro |
| `Microsoft Motion Controller` | WMR headsets |
| `Meta Hand Tracking Aim` | Quest hand tracking |

### Input Action Binding

```csharp
using UnityEngine.InputSystem;
using UnityEngine.XR.OpenXR.Input;

public class OpenXRInputExample : MonoBehaviour
{
    // Use InputActionReference for cross-platform binding
    [SerializeField] private InputActionReference _gripAction;
    [SerializeField] private InputActionReference _triggerAction;
    [SerializeField] private InputActionReference _primaryButtonAction;

    private void OnEnable()
    {
        _gripAction.action.performed += OnGrip;
        _gripAction.action.canceled += OnGripReleased;
        _gripAction.action.Enable();

        _triggerAction.action.performed += OnTrigger;
        _triggerAction.action.Enable();
    }

    private void OnDisable()
    {
        _gripAction.action.performed -= OnGrip;
        _gripAction.action.canceled -= OnGripReleased;
        _triggerAction.action.performed -= OnTrigger;
    }

    private void OnGrip(InputAction.CallbackContext ctx)
    {
        float gripValue = ctx.ReadValue<float>();
        // Handle grip input (0-1 range)
    }

    private void OnGripReleased(InputAction.CallbackContext ctx)
    {
        // Handle grip release
    }

    private void OnTrigger(InputAction.CallbackContext ctx)
    {
        float triggerValue = ctx.ReadValue<float>();
        // Handle trigger input (0-1 range)
    }
}
```

## Action-Based Input System

### Default Input Actions Asset

Create or use the XR Interaction Toolkit default actions:

```
Assets/
  XR/
    DefaultInputActions.inputactions
```

**Common Actions**:
- `XRI LeftHand/Position` - Controller position
- `XRI LeftHand/Rotation` - Controller rotation
- `XRI LeftHand/Grip` - Grip button (float)
- `XRI LeftHand/Trigger` - Trigger button (float)
- `XRI LeftHand/Primary Button` - A/X button
- `XRI LeftHand/Thumbstick` - Thumbstick (Vector2)

### Binding Paths

```xml
<!-- OpenXR binding paths -->
<Binding path="<XRController>{LeftHand}/grip"/>
<Binding path="<XRController>{RightHand}/trigger"/>
<Binding path="<XRController>{LeftHand}/thumbstick"/>
<Binding path="<XRController>/devicePosition"/>
<Binding path="<XRController>/deviceRotation"/>

<!-- Eye tracking (if supported) -->
<Binding path="<EyeGaze>/pose"/>
```

## OpenXR Features (Extensions)

### Unity Feature Groups

Enable in Project Settings → XR Plug-in Management → OpenXR → Features:

| Feature | Purpose |
|---------|---------|
| Hand Tracking | XR Hands API support |
| Eye Gaze Interaction | Eye tracking input |
| Performance Settings | Foveated rendering |
| Mock Runtime | Editor testing |

### Meta Quest Features

```csharp
using UnityEngine.XR.OpenXR.Features.Meta;

// Enable Quest-specific features via OpenXR extensions
// These require the Meta OpenXR feature group

// Passthrough (Quest 2/Pro/3)
// Hand Tracking
// Eye Tracking (Pro/3)
// Face Tracking (Pro/3)
```

## Runtime Selection

### Automatic Runtime Detection

OpenXR automatically selects the active runtime:

```csharp
using UnityEngine.XR.OpenXR;

public class RuntimeInfo : MonoBehaviour
{
    private void Start()
    {
        var runtime = OpenXRRuntime.name;
        var version = OpenXRRuntime.version;

        Debug.Log($"OpenXR Runtime: {runtime} v{version}");

        // Example outputs:
        // "Oculus" for Quest/Rift
        // "SteamVR/OpenXR" for SteamVR
        // "Windows Mixed Reality Runtime" for WMR
    }
}
```

### Manual Runtime Override

```bash
# Windows: Set environment variable to force runtime
set XR_RUNTIME_JSON=C:\path\to\runtime.json

# Quest: Uses Oculus runtime by default
# SteamVR: Set SteamVR as default in SteamVR settings
```

## Tracking and Reference Spaces

### Reference Space Types

```csharp
using UnityEngine.XR;

public class TrackingOriginSetup : MonoBehaviour
{
    private void Start()
    {
        var xrOrigin = GetComponent<XROrigin>();

        // Floor-level tracking (standing/room-scale)
        xrOrigin.RequestedTrackingOriginMode = XROrigin.TrackingOriginMode.Floor;

        // Device-level tracking (seated)
        // xrOrigin.RequestedTrackingOriginMode = XROrigin.TrackingOriginMode.Device;
    }
}
```

### Boundary/Guardian

```csharp
using UnityEngine.XR;
using System.Collections.Generic;

public class BoundaryVisualization : MonoBehaviour
{
    private void CheckBoundary()
    {
        var boundaryPoints = new List<Vector3>();

        if (XRInputSubsystem.TryGetBoundaryPoints(boundaryPoints))
        {
            // Visualize play area boundary
            foreach (var point in boundaryPoints)
            {
                Debug.DrawLine(point, point + Vector3.up, Color.cyan);
            }
        }
    }
}
```

## Performance Extensions

### Foveated Rendering

```csharp
using UnityEngine.XR;

public class FoveationSettings : MonoBehaviour
{
    public enum FoveationLevel { None, Low, Medium, High }

    public void SetFoveationLevel(FoveationLevel level)
    {
        var displays = new List<XRDisplaySubsystem>();
        SubsystemManager.GetSubsystems(displays);

        foreach (var display in displays)
        {
            var mode = level switch
            {
                FoveationLevel.None => XRDisplaySubsystem.FocusedRenderingMode.Disabled,
                FoveationLevel.Low => XRDisplaySubsystem.FocusedRenderingMode.Low,
                FoveationLevel.Medium => XRDisplaySubsystem.FocusedRenderingMode.Medium,
                FoveationLevel.High => XRDisplaySubsystem.FocusedRenderingMode.High,
                _ => XRDisplaySubsystem.FocusedRenderingMode.Disabled
            };

            display.TrySetFocusedRenderingMode(mode);
        }
    }
}
```

## Cross-Platform Best Practices

1. **Use Action-Based Input**: Never hardcode button names
2. **Test Multiple Runtimes**: Verify on Oculus, SteamVR, WMR
3. **Handle Missing Features**: Check for extension support before use
4. **Abstract Platform Code**: Wrap platform-specific features in interfaces
5. **Default to OpenXR**: Only add vendor SDKs for specific features

### Platform Abstraction Pattern

```csharp
public interface IPlatformFeatures
{
    bool SupportsPassthrough { get; }
    bool SupportsEyeTracking { get; }
    bool SupportsHandTracking { get; }
    void EnablePassthrough();
}

public class QuestPlatformFeatures : IPlatformFeatures
{
    public bool SupportsPassthrough => true; // Quest 2/Pro/3
    public bool SupportsEyeTracking => IsQuestPro || IsQuest3;
    public bool SupportsHandTracking => true;

    public void EnablePassthrough()
    {
        // Quest-specific passthrough via Meta SDK
    }
}

public class SteamVRPlatformFeatures : IPlatformFeatures
{
    public bool SupportsPassthrough => false; // Device dependent
    public bool SupportsEyeTracking => HasEyeTrackingExtension;
    public bool SupportsHandTracking => HasHandTrackingExtension;

    public void EnablePassthrough()
    {
        throw new NotSupportedException("Passthrough not available");
    }
}
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| No controllers detected | Check interaction profile is enabled |
| Wrong tracking origin | Verify XROrigin tracking mode |
| Input not working | Ensure Input System package installed, actions enabled |
| Runtime not found | Install/enable platform runtime (Oculus app, SteamVR) |
| Extension not available | Check device/runtime support for feature |
