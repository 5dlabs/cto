---
name: three-js
description: Three.js performance optimization - 100+ rules for memory, rendering, geometry, materials, WebGPU, and WebXR.
---

# Three.js Best Practices

Comprehensive performance optimization guide for Three.js applications with 100+ rules across 17 categories.

## When to Apply

Reference these guidelines when:
- Setting up a new Three.js project
- Writing or reviewing Three.js code
- Optimizing performance or fixing memory leaks
- Working with custom shaders (GLSL or TSL)
- Implementing WebGPU features
- Building VR/AR experiences with WebXR

## Rule Categories by Priority

| Priority | Category | Impact | Prefix |
|----------|----------|--------|--------|
| 0 | Modern Setup & Imports | FUNDAMENTAL | `setup-` |
| 1 | Memory Management & Dispose | CRITICAL | `memory-` |
| 2 | Render Loop Optimization | CRITICAL | `render-` |
| 3 | Geometry & Buffer Management | HIGH | `geometry-` |
| 4 | Material & Texture Optimization | HIGH | `material-` |
| 5 | Lighting & Shadows | MEDIUM-HIGH | `lighting-` |
| 6 | Scene Graph Organization | MEDIUM | `scene-` |
| 7-8 | Shader Best Practices | MEDIUM | `shader-`, `tsl-` |
| 9-17 | Loading, Camera, Animation, Physics, WebXR, Audio, Mobile, Production, Debug | VARIES | - |

## Quick Reference

### Modern Import Maps

```html
<script type="importmap">
{
  "imports": {
    "three": "https://cdn.jsdelivr.net/npm/three@0.182.0/build/three.module.js",
    "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.182.0/examples/jsm/",
    "three/tsl": "https://cdn.jsdelivr.net/npm/three@0.182.0/build/three.tsl.js"
  }
}
</script>
```

### Memory Management (CRITICAL)

**Always dispose resources:**

```javascript
function disposeObject(obj) {
  if (obj.geometry) obj.geometry.dispose();
  if (obj.material) {
    if (Array.isArray(obj.material)) {
      obj.material.forEach(m => m.dispose());
    } else {
      obj.material.dispose();
    }
  }
}

// Recursive disposal for hierarchies
function disposeHierarchy(obj) {
  obj.traverse(child => disposeObject(child));
}
```

### Render Loop (CRITICAL)

```javascript
// Use setAnimationLoop, not manual RAF
renderer.setAnimationLoop(animate);

// Never allocate in render loop
const _tempVector = new THREE.Vector3();  // Reuse outside loop

function animate(time) {
  // Use cached objects
  _tempVector.set(0, 1, 0);
  
  // Use delta time for animations
  const delta = clock.getDelta();
  mixer.update(delta);
  
  renderer.render(scene, camera);
}
```

### Geometry (HIGH)

```javascript
// Use InstancedMesh for identical objects
const mesh = new THREE.InstancedMesh(geometry, material, count);
const matrix = new THREE.Matrix4();

for (let i = 0; i < count; i++) {
  matrix.setPosition(positions[i]);
  mesh.setMatrixAt(i, matrix);
}
mesh.instanceMatrix.needsUpdate = true;
```

### Materials & Textures (HIGH)

```javascript
// Reuse materials
const sharedMaterial = new THREE.MeshStandardMaterial({ color: 0xff0000 });

// Use compressed textures
const ktx2Loader = new KTX2Loader()
  .setTranscoderPath('path/to/basis/')
  .detectSupport(renderer);

// Power-of-two texture dimensions
// 512x512, 1024x1024, 2048x2048
```

### Mobile Optimization

```javascript
const isMobile = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);

// Limit pixel ratio
renderer.setPixelRatio(Math.min(window.devicePixelRatio, isMobile ? 1.5 : 2));

// Reduce shadow map size on mobile
if (isMobile) {
  light.shadow.mapSize.width = 512;
  light.shadow.mapSize.height = 512;
}
```

### TSL (Three.js Shading Language)

```javascript
import { texture, uv, color, time, sin } from 'three/tsl';

const material = new THREE.MeshStandardNodeMaterial();
material.colorNode = texture(map).mul(color(0xff0000));
material.colorNode = color(0x00ff00).mul(sin(time).mul(0.5).add(0.5));
```

## Key Patterns

| Pattern | Benefit |
|---------|---------|
| Use InstancedMesh | 100x draw calls → 1 draw call |
| Merge static geometries | Reduce draw calls |
| Use LOD (Level of Detail) | Performance at distance |
| Enable frustum culling | Skip off-screen objects |
| Use texture atlases | Reduce material switches |
| Bake lighting | Eliminate runtime shadows |

## Anti-Patterns

| Anti-Pattern | Fix |
|--------------|-----|
| Creating objects in render loop | Cache and reuse |
| Not disposing resources | Always call .dispose() |
| Using BufferGeometry constructor | Use specific geometry classes |
| Too many lights | Limit to 3-4 dynamic lights |
| High pixel ratio on mobile | Cap at 1.5-2 |

## Attribution

Based on [emalorenzo/three-agent-skills](https://github.com/emalorenzo/three-agent-skills) three-best-practices - 53+ installs. Official guidelines from Three.js `llms` branch maintained by mrdoob.
