# 👾Multi-Agent 🎮 FPS Gym Environment with 🏋️ bevy_rl

[![Crates.io](https://img.shields.io/crates/v/bevy_quadruped_neural_control.svg)](https://crates.io/crates/bevy_quadruped_neural_control)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/bevyengine/bevy#license)
[![Rust](https://github.com/stillonearth/bevy_quadruped_neural_control/workflows/CI/badge.svg)](https://github.com/stillonearth/bevy_quadruped_neural_control/actions)

##

https://user-images.githubusercontent.com/97428129/192408835-3a2857bf-ee6a-4213-b469-d0af0a1fc75b.mp4

## Project Description

This project is an attempt to build minimal multi-agent FPS game with Bevy Game Engine (0.8) and train AI agent with DQN.

## Project Goals

- Create a multi-agent gym environment
- Create an example project for bevy_rl

## Environment Description

- Deathmatch free-for-all
- Agents spawn as red spheres
- 1-hit kill
- +10 reward on kill

## Implementation Details

- [bevy](https://bevyengine.org/) is a free game engine written in Rust
- [bevy_rl](https://github.com/stillonearth/bevy_rl) is a plugin for Bevy that implements OpenAI Gym interface

## Usage

- follow bevy's [setup guide](https://bevyengine.org/learn/book/getting-started/setup/) to set up Rust
- build an environment with `cargo build --release`; run environment with `./target/release/bevy_rl_shooter  --mode train`
- `python/env.py` implements a python wrapper for an environment
- `python/DQN.ipynb` is a basic DQN agent trained on the environment
