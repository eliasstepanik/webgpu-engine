// Outline shader for selection highlighting

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct ObjectUniform {
    model: mat4x4<f32>,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> object: ObjectUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Scale the vertex position along the normal to create an outline
    let scale_factor = 1.02; // 2% larger
    let scaled_position = input.position + input.normal * 0.02;
    
    let world_position = object.model * vec4<f32>(scaled_position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Use the object color for the outline (typically bright/contrasting color)
    return object.color;
}