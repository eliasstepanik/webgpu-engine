## FEATURE:
Extend large world coordinates to support Milky Way galaxy scale (100,000+ light-years)

## EXAMPLES:
.claude/prp/large-world-coordinates.md – existing planetary-scale coordinate implementation
examples/scene_demo.rs – demonstrates scene loading with transform components

## DOCUMENTATION:
https://en.wikipedia.org/wiki/Double-precision_floating-point_format – float64 max value ~1.8×10^308
https://en.wikipedia.org/wiki/Milky_Way – galaxy diameter 100,000 light-years (~10^21 meters)
https://imagine.gsfc.nasa.gov/features/cosmic/milkyway_info.html – NASA Milky Way measurements

## OTHER CONSIDERATIONS:
- Current implementation targets >1 million units, needs extension to ~10^21 meters
- Float64 max value (1.8×10^308) can theoretically handle galaxy scale
- 1 light-year = 9.46×10^15 meters, Milky Way = ~9.46×10^20 meters diameter
- May need hierarchical coordinate systems (sectors/regions) for practical use
- Camera-relative rendering already implemented, needs scale validation
- Consider logarithmic depth buffer for extreme near/far ratios
- Performance implications of extreme coordinate transformations