# Unity VR Development Patterns

## Overview

Unity VR development using XR Interaction Toolkit (XRI) and OpenXR for cross-platform headset support.

## Core Architecture

### XR Origin Setup

```csharp
// Standard XR Origin hierarchy:
// XR Origin (XR Origin component)
//   └── Camera Offset (Transform only)
//       ├── Main Camera (Camera, TrackedPoseDriver, AudioListener)
//       ├── LeftHand Controller (XRController, XRDirectInteractor or XRRayInteractor)
//       └── RightHand Controller (XRController, XRDirectInteractor or XRRayInteractor)

using UnityEngine.XR.Interaction.Toolkit;
using UnityEngine.XR.Interaction.Toolkit.Inputs;

public class VRRigSetup : MonoBehaviour
{
    [SerializeField] private XROrigin _xrOrigin;
    [SerializeField] private ActionBasedController _leftController;
    [SerializeField] private ActionBasedController _rightController;

    private void Awake()
    {
        // XR Origin handles tracking origin and camera offset
        _xrOrigin.RequestedTrackingOriginMode = XROrigin.TrackingOriginMode.Floor;
    }
}
```

### XR Interaction Toolkit Components

| Component | Purpose |
|-----------|---------|
| `XROrigin` | Tracks headset and transforms world space to tracking space |
| `XRController` | Binds controller input actions to XR device |
| `XRDirectInteractor` | Near-field interaction (grabbing objects within reach) |
| `XRRayInteractor` | Far-field interaction (pointing at distant objects/UI) |
| `XRSocketInteractor` | Snap-to-socket placement |
| `XRGrabInteractable` | Objects that can be picked up |
| `TeleportationArea/Anchor` | Teleport destinations |

### Interaction Setup

```csharp
using UnityEngine.XR.Interaction.Toolkit;

[RequireComponent(typeof(Rigidbody))]
[RequireComponent(typeof(XRGrabInteractable))]
public class GrabbableObject : MonoBehaviour
{
    private XRGrabInteractable _interactable;

    private void Awake()
    {
        _interactable = GetComponent<XRGrabInteractable>();

        // Configure grab behavior
        _interactable.movementType = XRBaseInteractable.MovementType.VelocityTracking;
        _interactable.throwOnDetach = true;
        _interactable.throwSmoothingDuration = 0.25f;
    }

    private void OnEnable()
    {
        _interactable.selectEntered.AddListener(OnGrabbed);
        _interactable.selectExited.AddListener(OnReleased);
    }

    private void OnDisable()
    {
        _interactable.selectEntered.RemoveListener(OnGrabbed);
        _interactable.selectExited.RemoveListener(OnReleased);
    }

    private void OnGrabbed(SelectEnterEventArgs args)
    {
        // Handle grab - e.g., haptic feedback
        if (args.interactorObject is XRBaseControllerInteractor controller)
        {
            controller.SendHapticImpulse(0.5f, 0.1f);
        }
    }

    private void OnReleased(SelectExitEventArgs args)
    {
        // Handle release
    }
}
```

## Locomotion Systems

### Teleportation (Comfort Default)

```csharp
using UnityEngine.XR.Interaction.Toolkit;

public class TeleportManager : MonoBehaviour
{
    [SerializeField] private TeleportationProvider _teleportProvider;
    [SerializeField] private XRRayInteractor _teleportInteractor;

    // Configure valid teleport surfaces with TeleportationArea components
    // Use TeleportationAnchor for specific landing points

    public void SetupTeleportation()
    {
        // Line visual during teleport aim
        var lineVisual = _teleportInteractor.GetComponent<XRInteractorLineVisual>();
        lineVisual.validColorGradient = CreateGradient(Color.green);
        lineVisual.invalidColorGradient = CreateGradient(Color.red);
    }

    private Gradient CreateGradient(Color color)
    {
        var gradient = new Gradient();
        gradient.SetKeys(
            new GradientColorKey[] { new(color, 0f), new(color, 1f) },
            new GradientAlphaKey[] { new(1f, 0f), new(0f, 1f) }
        );
        return gradient;
    }
}
```

### Continuous Movement (Opt-in)

```csharp
using UnityEngine.XR.Interaction.Toolkit;

public class ContinuousMovementSetup : MonoBehaviour
{
    [SerializeField] private ContinuousMoveProvider _moveProvider;
    [SerializeField] private ContinuousTurnProvider _turnProvider;
    [SerializeField] private TunnelingVignetteController _vignette;

    public void ConfigureComfortSettings()
    {
        // Snap turn by default (more comfortable)
        var snapTurn = GetComponent<SnapTurnProvider>();
        snapTurn.turnAmount = 45f; // 45 or 30 degree increments

        // If using continuous turn, apply vignette
        _turnProvider.turnSpeed = 60f;

        // Enable vignette during movement
        _vignette.defaultParameters.apertureSize = 0.7f;
        _vignette.defaultParameters.featheringEffect = 0.1f;
    }
}
```

## Hand Tracking

### XR Hands Integration

```csharp
using UnityEngine.XR.Hands;

public class HandTrackingManager : MonoBehaviour
{
    private XRHandSubsystem _handSubsystem;

    private void Start()
    {
        var subsystems = new List<XRHandSubsystem>();
        SubsystemManager.GetSubsystems(subsystems);

        if (subsystems.Count > 0)
        {
            _handSubsystem = subsystems[0];
            _handSubsystem.updatedHands += OnHandsUpdated;
        }
    }

    private void OnHandsUpdated(XRHandSubsystem subsystem,
        XRHandSubsystem.UpdateSuccessFlags updateSuccessFlags,
        XRHandSubsystem.UpdateType updateType)
    {
        if ((updateSuccessFlags & XRHandSubsystem.UpdateSuccessFlags.LeftHandRootPose) != 0)
        {
            ProcessHand(subsystem.leftHand);
        }

        if ((updateSuccessFlags & XRHandSubsystem.UpdateSuccessFlags.RightHandRootPose) != 0)
        {
            ProcessHand(subsystem.rightHand);
        }
    }

    private void ProcessHand(XRHand hand)
    {
        // Access joint data
        if (hand.GetJoint(XRHandJointID.IndexTip).TryGetPose(out Pose indexTip))
        {
            // Use index fingertip position
        }
    }
}
```

## UI in VR

### World Space Canvas

```csharp
using UnityEngine.UI;
using UnityEngine.XR.Interaction.Toolkit.UI;

public class VRUISetup : MonoBehaviour
{
    [SerializeField] private Canvas _worldCanvas;
    [SerializeField] private XRUIInputModule _inputModule;

    private void Awake()
    {
        // World space canvas for VR
        _worldCanvas.renderMode = RenderMode.WorldSpace;

        // Position 1.5-2m from user
        _worldCanvas.transform.localPosition = new Vector3(0, 1.5f, 2f);

        // Scale for readability (typically 0.001-0.002)
        _worldCanvas.transform.localScale = Vector3.one * 0.001f;

        // Use TrackedDeviceGraphicRaycaster for XR input
        var raycaster = _worldCanvas.GetComponent<TrackedDeviceGraphicRaycaster>();
        if (raycaster == null)
        {
            _worldCanvas.gameObject.AddComponent<TrackedDeviceGraphicRaycaster>();
        }
    }
}
```

## Performance Optimization

### Best Practices

1. **Frame Budget**: Target 11ms for 90fps, 13.8ms for 72fps
2. **Draw Calls**: Keep under 100 for mobile VR
3. **Triangles**: ~100k for Quest, ~1M for PCVR
4. **Texture Size**: 1024x1024 max for Quest
5. **Shader Complexity**: Use mobile-optimized shaders on Quest

### Foveated Rendering

```csharp
using UnityEngine.XR;

public class FoveatedRenderingSetup : MonoBehaviour
{
    private void Start()
    {
        // Enable foveated rendering on supported devices
        var displays = new List<XRDisplaySubsystem>();
        SubsystemManager.GetSubsystems(displays);

        foreach (var display in displays)
        {
            if (display.TrySetFocusedRenderingMode(XRDisplaySubsystem.FocusedRenderingMode.Low))
            {
                Debug.Log("Foveated rendering enabled");
            }
        }
    }
}
```

## Testing VR

### Play Mode Testing

```csharp
using NUnit.Framework;
using UnityEngine.TestTools;

public class VRInteractionTests
{
    [UnityTest]
    public IEnumerator GrabInteractable_WhenSelected_BecomesAttached()
    {
        // Setup
        var interactable = CreateTestGrabbable();
        var interactor = CreateMockInteractor();

        // Act
        interactor.StartManualInteraction(interactable);
        yield return null;

        // Assert
        Assert.IsTrue(interactable.isSelected);
        Assert.AreEqual(interactor, interactable.firstInteractorSelecting);
    }
}
```

## Platform-Specific Notes

| Platform | Considerations |
|----------|----------------|
| Meta Quest | Use Vulkan, texture compression ASTC, eye tracking via Meta SDK |
| SteamVR | OpenVR fallback for older headsets, skeletal hand input |
| Pico | Similar to Quest, use Pico SDK for platform features |
| Vive Focus | Use Wave SDK for standalone features |
