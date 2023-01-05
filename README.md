# 👾Multi-Agent 🎮 FPS Gym Environment with 🏋️ bevy_rl

[![Crates.io](https://img.shields.io/crates/v/bevy_rl_shooter.svg)](https://crates.io/crates/bevy_rl_shooter)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/bevyengine/bevy#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_rl_shooter.svg)](https://crates.io/crates/bevy_rl_shooter)
[![Rust](https://github.com/stillonearth/bevy_rl_shooter/workflows/CI/badge.svg)](https://github.com/stillonearth/bevy_rl_shooter/actions)

##

https://user-images.githubusercontent.com/97428129/192408835-3a2857bf-ee6a-4213-b469-d0af0a1fc75b.mp4

## Project Description

This is a basic multi-agent gym environment for bevy_rl. It is a deathmatch free-for-all environment where agents spawn as red spheres and get +10 reward on kill. The environment is implemented in Rust using [bevy](https://bevyengine.org/) game engine and [bevy_rl](https://github.com/stillonearth/bevy_rl) plugin.

It implements very basics of a multi-agent environment. It is a good starting point for creating more complex environments.

- Random initialization of agents
- REST API for controlling agents (including state, reward and camera pixels)
- REST API to reset an environment

You can wrap the environment with a python wrapper and use it with OpenAI Gym interface. (example in `python/bevy_rl_rest_api.ipynb`)

## Environment Description

- 16 agents spawn in a random position
- Agents can move and rotate
- Environment pauses every 0.1 second to fetch control commands from REST API
- Reward: +10 on kill

## Usage

- follow bevy's [setup guide](https://bevyengine.org/learn/book/getting-started/setup/) to set up Rust
- build an environment with `cargo build +nightly --release`;
- run environment with `./target/release/bevy_rl_shooter  --mode train`
- `python/env.py` implements a python wrapper for an environment
- `python/bevy_rl_rest_api.ipynb` illustrates how to use the wrapper
