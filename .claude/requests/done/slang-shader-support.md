## FEATURE:

Implement support for the Slang shading language in the WebGPU template engine. This includes:

1. **Slang Compiler Integration**: Add the ability to compile Slang shaders to SPIR-V for WebGPU compatibility
2. **Shader Management System**: Create a flexible shader loading and compilation system that supports both WGSL and Slang
3. **Hot-Reload Support**: Implement runtime shader reloading for development workflow
4. **Shader Abstraction Layer**: Design an abstraction that allows easy switching between shader languages
5. **Pipeline Management**: Extend the current pipeline system to support multiple shader types and configurations

## EXAMPLES:

Example Slang shader structure:
```slang
// basic.slang
struct VertexInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
}

struct VertexOutput {
    float4 position : SV_Position;
    float3 worldNormal : NORMAL;
    float2 uv : TEXCOORD0;
}

struct CameraUniforms {
    float4x4 viewProjection;
}

struct ObjectUniforms {
    float4x4 model;
    float4 color;
}

[shader("vertex")]
VertexOutput vertexMain(VertexInput input,
                       ConstantBuffer<CameraUniforms> camera,
                       ConstantBuffer<ObjectUniforms> object) {
    VertexOutput output;
    float4 worldPos = mul(object.model, float4(input.position, 1.0));
    output.position = mul(camera.viewProjection, worldPos);
    output.worldNormal = normalize(mul(float3x3(object.model), input.normal));
    output.uv = input.uv;
    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input,
                   ConstantBuffer<ObjectUniforms> object) : SV_Target {
    float3 lightDir = normalize(float3(0.5, -0.7, -0.5));
    float diffuse = max(dot(input.worldNormal, -lightDir), 0.0);
    float3 color = object.color.rgb * (0.3 + 0.7 * diffuse);
    return float4(color, object.color.a);
}
```

## DOCUMENTATION:

1. **Slang Language Reference**: https://github.com/shader-slang/slang
2. **slang-rs Rust Bindings**: https://github.com/FloatyMonkey/slang-rs
3. **WebGPU Shader Requirements**: https://www.w3.org/TR/webgpu/#shader-modules
4. **SPIR-V Specification**: https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html
5. **Current WGSL Implementation**: `/engine/src/shaders/basic.wgsl`

## OTHER CONSIDERATIONS:

1. **Slang SDK Installation**: The slang-rs crate requires the Slang SDK to be installed. This can be obtained through:
   - Vulkan SDK (recommended)
   - Direct download from Slang releases
   - The build system should detect and validate Slang installation

2. **Shader Compilation Pipeline**:
   - Slang → SPIR-V → WebGPU shader module
   - Need to handle compilation errors gracefully
   - Cache compiled shaders to avoid recompilation

3. **Backward Compatibility**:
   - Must maintain support for existing WGSL shaders
   - The shader system should auto-detect shader language based on file extension
   - Existing code using `BASIC_SHADER` constant should continue to work

4. **Performance Considerations**:
   - Shader compilation can be expensive - implement caching
   - Hot-reload should only trigger for changed shaders
   - Consider async compilation to avoid blocking the render thread

5. **Error Handling**:
   - Slang compilation errors should be clearly reported
   - Fallback to a default shader if compilation fails
   - Log shader compilation times for performance monitoring

6. **Testing Requirements**:
   - Unit tests for shader compilation
   - Integration tests with actual rendering
   - Performance benchmarks comparing WGSL vs Slang compiled shaders

7. **Future Extensions**:
   - Support for Slang's advanced features (interfaces, generics)
   - Integration with Slang's automatic differentiation capabilities
   - Support for compute shaders
   - Shader parameter introspection and automatic bind group layout generation

8. **Build Configuration**:
   - Make Slang support optional via a Cargo feature flag
   - Document how to set up the development environment with Slang SDK
   - CI/CD pipeline updates to include Slang SDK