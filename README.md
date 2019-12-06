# Rendology
[![Docs Status](https://docs.rs/rendology/badge.svg)](https://docs.rs/rendology)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/leod/rendology/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rendology.svg)](https://crates.io/crates/rendology)

Rendology is a [Glium](https://github.com/glium/glium)-based 3D rendering library. It is not, however, a game engine.

:warning: Rendology is in a very early stage; it is undocumented and subject to frequent changes. Using this crate is not (yet) recommended! :warning:

## Features
Rendology features amateur-grade implementations of:

- Shadow mapping
- Deferred shading
- A glow effect
- FXAA
- Instanced rendering

Each of the rendering effects can be turned on and off separately.

Rendology makes it easy to integrate custom scene shaders into the rendering pipeline.
