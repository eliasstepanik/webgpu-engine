// Basic vertex and fragment shaders for 3D rendering

// Camera uniform buffer containing view-projection matrix
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

// Object uniform buffer containing model matrix and material color
struct ObjectUniform {
    model: mat4x4<f32>,
    color: vec4<f32>,
};

// Bind groups
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> object: ObjectUniform;

// Vertex input structure
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

// Vertex output / Fragment input structure
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

// Vertex shader
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_position = object.model * vec4<f32>(in.position, 1.0);
    out.world_position = world_position.xyz;
    
    // Transform normal to world space (using normal matrix - transpose of inverse model)
    // For now, assuming uniform scale, so we can use the model matrix directly
    let world_normal = normalize((object.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_normal = world_normal;
    
    // Pass through UV coordinates
    out.uv = in.uv;
    
    // Pass through material color
    out.color = object.color;
    
    // Calculate clip position
    out.clip_position = camera.view_proj * world_position;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional light
    let light_dir = normalize(vec3<f32>(0.5, -1.0, -0.3));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let ambient_strength = 0.1;
    
    // Calculate diffuse lighting
    let diff = max(dot(in.world_normal, -light_dir), 0.0);
    let diffuse = diff * light_color;
    
    // Calculate ambient lighting
    let ambient = ambient_strength * light_color;
    
    // Combine lighting with material color
    let result = (ambient + diffuse) * in.color.rgb;
    
    return vec4<f32>(result, in.color.a);
}