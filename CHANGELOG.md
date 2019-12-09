# Changelog
## Version 0.3.0 (2019-12-10)
- Allow passing draw parameters to shadow pass ([#3](https://github.com/leod/rendology/pull/3))
- Overhaul `ToUniforms` to allow lifetimes ([#3](https://github.com/leod/rendology/pull/3))
- Add textured cube example ([#3](https://github.com/leod/rendology/pull/3))
- Fix bug where glow output texture was not set ([#3](https://github.com/leod/rendology/pull/3))
- `impl_instance_input` and `impl_uniform_input` now take the actual rust types ([#3](https://github.com/leod/rendology/pull/3))
- Reworked macro syntax ([#4](https://github.com/leod/rendology/pull/4))
- More explicit names for built-in uniforms ([#4](https://github.com/leod/rendology/pull/4))
- Make more shading parameters configurable ([#5](https://github.com/leod/rendology/pull/5))

## Version 0.2.0 (2019-12-06)
- Replace `ToVertex` trait by `InstanceInput`
- Allow specifying the `InstancingMode` for scene passes
- Implement uniform-based drawing of `RenderList`

## Version 0.1.0 (2019-12-06)
- Initial version
