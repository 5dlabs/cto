using UnityEngine;
using UnityEngine.XR.Interaction.Toolkit;
using System.Collections.Generic;

/// <summary>
/// Player controller for XR interaction.
/// Test fixture for Vex agent detection.
/// </summary>
public class PlayerController : MonoBehaviour
{
    public float moveSpeed = 5.0f;  // Subtle: magic number
    public float rotationSpeed = 100.0f;  // Subtle: magic number
    
    private Rigidbody rb;
    private List<GameObject> trackedObjects = new List<GameObject>();
    
    // Subtle: using Update for physics
    void Update()
    {
        float horizontal = Input.GetAxis("Horizontal");
        float vertical = Input.GetAxis("Vertical");
        
        // Subtle: direct transform manipulation in Update
        transform.Translate(new Vector3(horizontal, 0, vertical) * moveSpeed * Time.deltaTime);
        
        // Subtle: Find in Update (performance issue)
        GameObject target = GameObject.Find("Target");
        if (target != null)
        {
            transform.LookAt(target.transform);
        }
        
        // Subtle: GetComponent in Update (should cache)
        var xrController = GetComponent<XRController>();
        if (xrController != null)
        {
            ProcessXRInput(xrController);
        }
    }
    
    void ProcessXRInput(XRController controller)
    {
        // Subtle: empty method body
    }
    
    // Subtle: using OnGUI for HUD (should use new UI system)
    void OnGUI()
    {
        GUI.Label(new Rect(10, 10, 100, 20), "Speed: " + moveSpeed);
    }
    
    // Subtle: public field modification
    public void SetSpeed(float speed)
    {
        moveSpeed = speed;  // No validation
    }
    
    // Subtle: comparing floating point with ==
    public bool IsStationary()
    {
        return rb.velocity.magnitude == 0.0f;
    }
    
    // Subtle: string concatenation in loop
    public string GetTrackedObjectNames()
    {
        string result = "";
        foreach (var obj in trackedObjects)
        {
            result = result + obj.name + ", ";  // Should use StringBuilder
        }
        return result;
    }
    
    void OnDestroy()
    {
        // Subtle: not cleaning up event subscriptions
        trackedObjects.Clear();
    }
}
