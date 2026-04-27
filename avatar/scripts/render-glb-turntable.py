"""Render a GLB into still previews and a short turntable MP4 with Blender.

The direct-mesh mode is intentionally small and dependency-light. It bypasses
Blender's glTF importer for first-pass visual checks when generated GLBs trigger
pathological importer memory use. It still applies the glTF Y-up to Blender Z-up
axis conversion and preserves vertex colors so the default front still is useful
for human review.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import struct
import sys
from pathlib import Path

import bpy
from mathutils import Vector


COMPONENT_FORMATS = {
    5120: ("b", 1),
    5121: ("B", 1),
    5122: ("h", 2),
    5123: ("H", 2),
    5125: ("I", 4),
    5126: ("f", 4),
}

TYPE_COUNTS = {
    "SCALAR": 1,
    "VEC2": 2,
    "VEC3": 3,
    "VEC4": 4,
    "MAT4": 16,
}

FALSE_VALUES = {"0", "false", "no", "off"}


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def env_flag(name: str, default: bool = False) -> bool:
    value = os.environ.get(name)
    if value is None:
        return default
    return value.strip().lower() not in FALSE_VALUES


def clamp_color(value: float) -> float:
    value = float(value)
    if value > 1.0:
        value /= 255.0
    return max(0.0, min(1.0, value))


def set_engine() -> None:
    preferred = os.environ.get("BLENDER_RENDER_ENGINE")
    engines = [preferred] if preferred else []
    engines.extend(["BLENDER_EEVEE_NEXT", "BLENDER_EEVEE", "BLENDER_WORKBENCH", "CYCLES"])
    for engine in engines:
        if not engine:
            continue
        try:
            bpy.context.scene.render.engine = engine
            return
        except TypeError:
            continue
    raise RuntimeError("No supported Blender render engine found")


def imported_bounds() -> tuple[Vector, Vector, float]:
    mesh_objects = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not mesh_objects:
        raise RuntimeError("GLB imported successfully but contains no mesh objects")

    mins = Vector((math.inf, math.inf, math.inf))
    maxs = Vector((-math.inf, -math.inf, -math.inf))
    for obj in mesh_objects:
        for corner in obj.bound_box:
            world = obj.matrix_world @ Vector(corner)
            mins.x = min(mins.x, world.x)
            mins.y = min(mins.y, world.y)
            mins.z = min(mins.z, world.z)
            maxs.x = max(maxs.x, world.x)
            maxs.y = max(maxs.y, world.y)
            maxs.z = max(maxs.z, world.z)

    center = (mins + maxs) * 0.5
    size = maxs - mins
    max_dim = max(size.x, size.y, size.z)
    if max_dim <= 0:
        raise RuntimeError("Imported GLB has invalid zero-size bounds")
    return center, size, max_dim


def look_at(obj: bpy.types.Object, target: Vector) -> None:
    direction = target - obj.location
    obj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()


def setup_world() -> None:
    set_engine()
    scene = bpy.context.scene
    scene.render.resolution_x = int(os.environ.get("BLENDER_RENDER_WIDTH", "1280"))
    scene.render.resolution_y = int(os.environ.get("BLENDER_RENDER_HEIGHT", "720"))
    scene.render.fps = 24
    if scene.render.engine == "BLENDER_WORKBENCH":
        scene.display.shading.color_type = "VERTEX"
    scene.view_settings.view_transform = "Filmic"
    scene.view_settings.look = "Medium High Contrast"
    scene.world = scene.world or bpy.data.worlds.new("World")
    scene.world.color = (0.45, 0.45, 0.45)


def parse_glb(path: Path) -> tuple[dict, bytes]:
    data = path.read_bytes()
    magic, version, declared_length = struct.unpack_from("<4sII", data, 0)
    if magic != b"glTF" or version != 2:
        raise RuntimeError(f"{path} is not a glTF 2.0 binary")
    if declared_length != len(data):
        raise RuntimeError(f"{path} length mismatch: header={declared_length} actual={len(data)}")

    json_doc = None
    binary_chunk = None
    offset = 12
    while offset + 8 <= len(data):
        chunk_length, chunk_type = struct.unpack_from("<II", data, offset)
        offset += 8
        chunk = data[offset : offset + chunk_length]
        offset += chunk_length
        chunk_tag = chunk_type.to_bytes(4, "little")
        if chunk_tag == b"JSON":
            json_doc = json.loads(chunk.decode("utf-8"))
        elif chunk_tag == b"BIN\x00":
            binary_chunk = chunk

    if json_doc is None or binary_chunk is None:
        raise RuntimeError(f"{path} is missing JSON or BIN chunks")
    return json_doc, binary_chunk


def read_accessor(json_doc: dict, binary_chunk: bytes, accessor_index: int) -> list[tuple]:
    accessor = json_doc["accessors"][accessor_index]
    buffer_view = json_doc["bufferViews"][accessor["bufferView"]]
    component_format, component_size = COMPONENT_FORMATS[accessor["componentType"]]
    component_count = TYPE_COUNTS[accessor["type"]]
    accessor_offset = accessor.get("byteOffset", 0)
    view_offset = buffer_view.get("byteOffset", 0)
    stride = buffer_view.get("byteStride", component_size * component_count)
    count = accessor["count"]
    fmt = "<" + (component_format * component_count)
    item_size = struct.calcsize(fmt)
    if item_size > stride:
        raise RuntimeError("Accessor item size exceeds bufferView stride")

    base = view_offset + accessor_offset
    values = []
    for index in range(count):
        start = base + index * stride
        values.append(struct.unpack_from(fmt, binary_chunk, start))
    return values


def import_glb_mesh_direct(path: Path) -> None:
    json_doc, binary_chunk = parse_glb(path)
    primitive = json_doc["meshes"][0]["primitives"][0]
    positions = read_accessor(json_doc, binary_chunk, primitive["attributes"]["POSITION"])
    indices = [item[0] for item in read_accessor(json_doc, binary_chunk, primitive["indices"])]
    colors_index = primitive["attributes"].get("COLOR_0")
    colors = read_accessor(json_doc, binary_chunk, colors_index) if colors_index is not None else None
    if len(indices) % 3 != 0:
        raise RuntimeError("Only triangle-list GLB primitives are supported for direct import")
    faces = [tuple(indices[index : index + 3]) for index in range(0, len(indices), 3)]
    if env_flag("BLENDER_DIRECT_MESH_Y_UP_TO_Z_UP", default=True):
        vertices = [(float(x), -float(z), float(y)) for x, y, z in positions]
    else:
        vertices = [(float(x), float(y), float(z)) for x, y, z in positions]

    mesh = bpy.data.meshes.new(path.stem)
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(path.stem, mesh)
    bpy.context.collection.objects.link(obj)

    if colors:
        color_attribute = mesh.color_attributes.new(name="Color", type="BYTE_COLOR", domain="CORNER")
        for polygon in mesh.polygons:
            for loop_index in polygon.loop_indices:
                vertex_index = mesh.loops[loop_index].vertex_index
                r, g, b = colors[vertex_index][:3]
                color_attribute.data[loop_index].color = (clamp_color(r), clamp_color(g), clamp_color(b), 1.0)
        mesh.color_attributes.active_color = color_attribute
        mesh.color_attributes.render_color_index = mesh.color_attributes.find(color_attribute.name)

        material = bpy.data.materials.new("Direct_GLB_Vertex_Color_Material")
        material.use_nodes = True
        nodes = material.node_tree.nodes
        bsdf = nodes.get("Principled BSDF")
        if bsdf is not None:
            color_node = nodes.new(type="ShaderNodeVertexColor")
            color_node.layer_name = "Color"
            material.node_tree.links.new(color_node.outputs["Color"], bsdf.inputs["Base Color"])
            bsdf.inputs["Roughness"].default_value = 0.85
    else:
        material = bpy.data.materials.new("Direct_GLB_Check_Material")
        material.diffuse_color = (0.72, 0.72, 0.72, 1.0)
    mesh.materials.append(material)


def add_lighting(center: Vector, max_dim: float) -> None:
    bpy.ops.object.light_add(type="AREA", location=(center.x - max_dim, center.y - max_dim, center.z + max_dim * 2.5))
    key = bpy.context.object
    key.name = "Key_Area"
    key.data.energy = 800
    key.data.size = max_dim * 2.0
    look_at(key, center)

    bpy.ops.object.light_add(type="POINT", location=(center.x + max_dim, center.y + max_dim, center.z + max_dim))
    fill = bpy.context.object
    fill.name = "Fill_Point"
    fill.data.energy = 120


def add_camera(center: Vector, max_dim: float) -> tuple[bpy.types.Object, float, float]:
    distance = max(max_dim * 2.8, 2.5)
    z_offset = max(max_dim * 0.25, 0.3)
    bpy.ops.object.camera_add(location=(center.x, center.y - distance, center.z + z_offset))
    camera = bpy.context.object
    camera.name = "Turntable_Camera"
    camera.data.lens = 55
    camera.data.clip_end = max(distance * 10, 1000)
    look_at(camera, center)
    bpy.context.scene.camera = camera
    return camera, distance, z_offset


def render_still(camera: bpy.types.Object, center: Vector, location: tuple[float, float, float], output: Path) -> None:
    camera.location = location
    look_at(camera, center)
    bpy.context.scene.render.filepath = str(output)
    bpy.ops.render.render(write_still=True)


def render_turntable(camera: bpy.types.Object, center: Vector, distance: float, z_offset: float, output: Path) -> None:
    target = bpy.data.objects.new("Turntable_Target", None)
    bpy.context.collection.objects.link(target)
    target.location = center

    camera.parent = target
    camera.location = (0, -distance, z_offset)
    constraint = camera.constraints.new(type="TRACK_TO")
    constraint.track_axis = "TRACK_NEGATIVE_Z"
    constraint.up_axis = "UP_Y"
    constraint.target = target

    frame_count = int(os.environ.get("BLENDER_TURNTABLE_FRAMES", "96"))
    scene = bpy.context.scene
    scene.frame_start = 1
    scene.frame_end = frame_count
    target.rotation_euler = (0, 0, 0)
    target.keyframe_insert(data_path="rotation_euler", frame=1)
    target.rotation_euler = (0, 0, math.tau)
    target.keyframe_insert(data_path="rotation_euler", frame=frame_count)
    for fcurve in target.animation_data.action.fcurves:
        for keyframe in fcurve.keyframe_points:
            keyframe.interpolation = "LINEAR"

    scene.render.filepath = str(output)
    scene.render.image_settings.file_format = "FFMPEG"
    scene.render.ffmpeg.format = "MPEG4"
    scene.render.ffmpeg.codec = "H264"
    scene.render.ffmpeg.constant_rate_factor = "MEDIUM"
    scene.render.ffmpeg.ffmpeg_preset = "GOOD"
    bpy.ops.render.render(animation=True)


def still_locations(
    import_mode: str, center: Vector, distance: float, z_offset: float
) -> tuple[tuple[float, float, float], tuple[float, float, float], tuple[float, float, float]]:
    if import_mode == "direct-mesh" and env_flag("BLENDER_DIRECT_MESH_Y_UP_TO_Z_UP", default=True):
        return (
            (center.x, center.y + distance, center.z + z_offset),
            (center.x + distance, center.y, center.z + z_offset),
            (center.x + distance * 0.7, center.y + distance * 0.7, center.z + z_offset),
        )
    return (
        (center.x, center.y - distance, center.z + z_offset),
        (center.x + distance, center.y, center.z + z_offset),
        (center.x + distance * 0.7, center.y - distance * 0.7, center.z + z_offset),
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", default=os.environ.get("BLENDER_GLB_INPUT"))
    parser.add_argument("--output-dir", default=os.environ.get("BLENDER_RENDER_OUTPUT_DIR"))
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    args = parser.parse_args(argv)
    if not args.input or not args.output_dir:
        parser.error("--input and --output-dir are required, or set BLENDER_GLB_INPUT and BLENDER_RENDER_OUTPUT_DIR")
    return args


def main() -> None:
    args = parse_args()
    input_path = Path(args.input)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    clear_scene()
    setup_world()
    import_mode = os.environ.get("BLENDER_GLB_IMPORT_MODE", "gltf")
    print(f"Importing {input_path} using {import_mode}", flush=True)
    if import_mode == "direct-mesh":
        import_glb_mesh_direct(input_path)
    else:
        bpy.ops.import_scene.gltf(filepath=str(input_path))
    print("Import complete", flush=True)

    center, size, max_dim = imported_bounds()
    add_lighting(center, max_dim)
    camera, distance, z_offset = add_camera(center, max_dim)

    front_location, side_location, three_quarter_location = still_locations(import_mode, center, distance, z_offset)
    render_still(camera, center, front_location, output_dir / "front.png")
    render_still(camera, center, side_location, output_dir / "side.png")
    render_still(camera, center, three_quarter_location, output_dir / "three_quarter.png")
    outputs = ["front.png", "side.png", "three_quarter.png"]
    if not env_flag("BLENDER_SKIP_TURNTABLE", default=False):
        render_turntable(camera, center, distance, z_offset, output_dir / "turntable.mp4")
        outputs.append("turntable.mp4")

    summary = output_dir / "summary.txt"
    summary.write_text(
        "\n".join(
            [
                f"input={input_path}",
                f"import_mode={import_mode}",
                f"bounds_size=({size.x:.4f},{size.y:.4f},{size.z:.4f})",
                f"max_dim={max_dim:.4f}",
                f"mesh_count={sum(1 for obj in bpy.context.scene.objects if obj.type == 'MESH')}",
                f"material_count={len(bpy.data.materials)}",
                "outputs=" + ",".join(outputs),
            ]
        )
        + "\n"
    )


if __name__ == "__main__":
    main()
