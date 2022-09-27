# Multi-Agent Minimal FPS Game Gym Environment with Bevy (v0.8) and bevy_rl

https://user-images.githubusercontent.com/97428129/192408835-3a2857bf-ee6a-4213-b469-d0af0a1fc75b.mp4

## Project Description

This project is an attempt to build minimal multi-agent FPS game with Bevy Game Engine (0.8) and train AI agent with DQN.

## Project Goals

- Create a multi-agent gym environment
- Create an example project for bevy_rl

## Implementation details

- [bevy])https://bevyengine.org/) is a free game engine written in Rust
- [bevy_rl](https://github.com/stillonearth/bevy_rl) is a plugin for Bevy that implements OpenAI Gym interface

## Usage

- `python/env.py` implements a python wrapper for an environment
- `python/DQN.ipynb` is a basic DQN agent trained on the environment
