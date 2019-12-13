# Rendology
[![Docs Status](https://docs.rs/rendology/badge.svg)](https://docs.rs/rendology)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/leod/rendology/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rendology.svg)](https://crates.io/crates/rendology)

![Ultimate Scale screenshot](https://leod.github.io/assets/rendology_hdr_0_4.png)

Rendology is a [Glium](https://github.com/glium/glium)-based 3D rendering library. It is not, however, a game engine.

:warning: Rendology is in a very early stage; it is undocumented and subject to frequent changes. Using this crate is not (yet) recommended! :warning:

There's a [blog post](https://leod.github.io/rust/gamedev/rendology/2019/12/13/introduction-to-rendology.html) on Rendology's architecture.

## Features
Rendology features amateur-grade implementations of:

- Shadow mapping
- Deferred shading
- A glow effect
- FXAA
- Instanced rendering

Each of the rendering effects can be turned on and off separately.

Rendology makes it easy to integrate custom scene shaders into the rendering pipeline.
