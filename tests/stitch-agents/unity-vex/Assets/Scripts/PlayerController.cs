/**
 * Stitch test fixture for Vex (Unity/C# XR agent)
 * 
 * This file contains intentional issues for testing remediation:
 * - Unused variables
 * - Missing null checks
 * - Performance issues (Find in Update)
 */

using UnityEngine;
using UnityEngine.XR.Interaction.Toolkit;

public class PlayerController : MonoBehaviour
{
    [SerializeField] private float moveSpeed = 5f;
    [SerializeField] private float rotationSpeed = 120f;
    
    // TODO: Intentional issue - unused field
    private int unusedCounter = 0;
    
    private CharacterController characterController;
    private XRDirectInteractor leftHand;
    private XRDirectInteractor rightHand;

    void Start()
    {
        // TODO: Intentional issue - no null check
        characterController = GetComponent<CharacterController>();
        
        // This will throw if component doesn't exist
        leftHand = GameObject.Find("LeftHand").GetComponent<XRDirectInteractor>();
        rightHand = GameObject.Find("RightHand").GetComponent<XRDirectInteractor>();
    }

    void Update()
    {
        // TODO: Intentional issue - Find in Update is expensive
        var camera = GameObject.Find("Main Camera");
        
        HandleMovement();
        HandleRotation();
    }

    private void HandleMovement()
    {
        float horizontal = Input.GetAxis("Horizontal");
        float vertical = Input.GetAxis("Vertical");

        // TODO: Intentional issue - magic number
        Vector3 movement = new Vector3(horizontal, 0, vertical) * moveSpeed * Time.deltaTime;
        characterController.Move(movement);
    }

    private void HandleRotation()
    {
        // TODO: Intentional issue - unused local variable
        float unusedAngle = 45f;
        
        float rotation = Input.GetAxis("Mouse X") * rotationSpeed * Time.deltaTime;
        transform.Rotate(0, rotation, 0);
    }

    // TODO: Intentional issue - public method with no documentation
    public void TeleportTo(Vector3 position)
    {
        characterController.enabled = false;
        transform.position = position;
        characterController.enabled = true;
    }
}
