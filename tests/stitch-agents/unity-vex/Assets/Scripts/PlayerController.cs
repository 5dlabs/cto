// Code smells for Vex (Unity/C#) to find:
// - Find calls in Update (expensive)
// - Not using object pooling
// - Magic numbers
// - Public fields instead of properties
// - Not null checking

using UnityEngine;
using UnityEngine.XR.Interaction.Toolkit;

public class PlayerController : MonoBehaviour
{
    // Should be [SerializeField] private with property
    public float moveSpeed = 5.0f;
    public GameObject bulletPrefab;
    
    // Magic numbers
    private float health = 100.0f;
    
    void Update()
    {
        // Expensive Find call every frame!
        var enemy = GameObject.Find("Enemy");
        
        // Magic numbers everywhere
        if (Input.GetKeyDown(KeyCode.Space))
        {
            // Instantiate in Update - should use object pooling
            var bullet = Instantiate(bulletPrefab, transform.position, Quaternion.identity);
            bullet.GetComponent<Rigidbody>().velocity = transform.forward * 50.0f;
            
            // Destroy after magic number seconds
            Destroy(bullet, 3.0f);
        }
        
        // Not checking if enemy is null
        float distance = Vector3.Distance(transform.position, enemy.transform.position);
        
        if (distance < 10.0f)
        {
            health -= 0.1f;
        }
    }
    
    // Unused method
    void OnTriggerEnter(Collider other)
    {
        // TODO: implement
    }
}
