## FEATURE:

Fix the remaining editor initialization issues on Windows. The editor now crashes with a scissor rect validation error instead of the texture panic. Additional issues may remain with the viewport rendering.

## EXAMPLES:

The crash occurs when running:
```
cargo run --features editor
```

Current error (after texture config fix):
```
wgpu error: Validation Error
Caused by:
  In RenderPass::end
    In a set_scissor_rect command
      Scissor Rect { x: 0, y: 0, w: 1920, h: 1080 } is not contained in the render target (1280, 720, 1)
```

Previous error (fixed):
```
thread 'main' panicked at imgui-wgpu-0.25.0\src\lib.rs:140:33:
called `Option::unwrap()` on a `None` value
```

## DOCUMENTATION:

- imgui-wgpu 0.25.0 documentation: https://docs.rs/imgui-wgpu/0.25.0/imgui_wgpu/
- The issue appears to be with the `from_raw_parts` method expecting a bind group when None is provided
- Review the exact API requirements for creating textures from existing wgpu resources

## OTHER CONSIDERATIONS:

- The fix has already been attempted by providing a RawTextureConfig, but the crash still occurs
- The issue might be with the bind_group parameter (currently None) which may be required
- Consider alternative approaches like using the simpler Texture::new() API if possible
- Test thoroughly on Windows after fixing to ensure viewport rendering works correctly