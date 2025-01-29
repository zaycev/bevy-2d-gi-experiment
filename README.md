<div align="center">

# 🔮 Magic Light 2D – experimental dynamic 2D global illumination system for Bevy Engine.

[![Build status](https://github.com/zaycev/bevy-magic-light-2d/actions/workflows/pr.yml/badge.svg?branch=main)](https://github.com/zaycev/bevy-magic-light-2d/actions)
[![dependency status](https://deps.rs/repo/github/zaycev/bevy-magic-light-2d/status.svg)](https://deps.rs/repo/github/zaycev/bevy-magic-light-2d)

</div>

<div alight="center">

[![Discord](https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0b5061df29d55a92d945_full_logo_blurple_RGB.svg)](https://discord.com/invite/tuXBTxF3W2) (ping me if it expires)

</div>

> 🚧 Beware! This repo is heavily under construction and a lot of things may change.

Experimental dynamic 2D global illumination system for Bevy, based on SDF ray-marching and screen space irradiance cache probes.

## Demo

```shell
cargo run --example minimal
cargo run --example krypta
cargo run --example movement
```

![Bevy Magic Light 2D – Demo](https://github.com/zaycev/bevy-magic-light-2d/blob/main/static/demo.gif?raw=true)

Implementation is based on several approaches:

- First pass computes SDF for all occluders and stores it in a texture with one channel.
- Second pass computes irradiance from direct light. We check amount of light received by probe taking into account occlusion which checked using SDF. The final contribution for each light source is computed using square falloff.
- Third pass computes secondary bounced light. The approach is similar to the second pass, but instead of direct light, it uses irradiance is sampled from the probes. We use exponential sampling to check how much of reflected light is received by probe. Same as in the second pass SDF is used to check occlusion and the final contribution is computed using square falloff.
- Finally, we combine results from the second and third passes combining cache from the previous eight frames. The we optionally filter the result using a edge-aware smoothing filter and apply Gamma correction.

The main performance gain is coming from computing value of only 1 / 64 number of pixels (for 8x8 probe size). The rest of the pixels are interpolated from the nearest probes.

### Usage

```shell
cargo run --example krypta
```

- WASD to control camera.
- SHIFT+LMC to place a light source.
- RMC to change color of light source.

## TODOs

**Optimizations**

- [ ] Use jump flood algorithm for calculating SDF.
- [ ] Precomputed noise.
- [ ] Guided sampling for secondary light.

**Features**

- [ ] Light can bounce from occluders.
- [ ] Arbitrary number of bounces via configuration.
- [x] Handle camera scale and rotation.
- [ ] Support multiple layers.
- [x] Expose settings instead of hardcoding them.
- [x] Support resize of targets.
- [ ] Support transparent occluders.
- [ ] Support color transfer from occluders.
- [x] Add inspector for GI settings.
- [ ] Add support for emissive materials and other types of light sources.

**Address limitations**

- [x] SDF for offscreen occluders.

**Others**

- [ ] Add examples and HUD explaining how to use example.

## Compatibility

| bevy   | bevy_magic_light_2d |
| ------ | ------------------- |
| `0.15` | `0.9`               |
| `0.14` | `0.8`               |

### License

```
Copyright 2022, 2023 Vladimir Zaytsev <vladimir@xyzw.io>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
