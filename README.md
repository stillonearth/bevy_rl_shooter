# [DRAFT] Minimal FPS Battle Royale Game Environment with Neural Agents using Variant of Deep-Q-Network

**Sergei Surovtsev** <<ssurovsev@gmail.com>>
<br />
February 2022
## Project Description

This project is an attempt to build minimal FPS game with Bevy Game Engine (0.6.0) and create AI agent to play it competitively with human player.

## Project Goals

* Implementing minimal FPS environment with Bevy Game Engine (0.6.0)
* Researching on feasibility of DQN for competitive FPS games

## Technical Formulation of Problem

* Implement FPS Gym Environment
* Train an AI agent

## Architecture

### Game

### Training Environment

Environment spawns a http server for programmatic interaction. API is available on http://localhost:7878/

Following API handles are exposed:

* **[GET]** `http://localhost:7878/screen.png` — A first-person view of the world by the agent
* **[POST]** `http://localhost:7878/step` — Perform a step and evaluate reward 
    <br />*POST BODY*: ACTION 
    <br />*RESPONSE*: ```json {is_terminated: false, reward: 10.0}```

## Mathematical Models

## Results

## Progress

Currently working

* [x] Collisions
* [x] Shooting
* [x] Map consturction
* [x] Enemies
* [x] Player health
* [x] Enemy movement
* [x] Enemy shooting via raycast
* [x] Gym environment

TODO:

* [ ] Wall textures

* [ ] Enemy AI with neural networks

## Known Bugs

* This environment uses unstable bevy version to enable rendering to a membuffer.
* Environment wouldn't do proper reset. In order to restart the environment during training you need to shut down the process and run it again. This should be resolved with bevy 0.7 release.

## Acknowledgements

* Inspired by [rustenstein](https://github.com/AdRoll/rustenstein) by AdRoll. Map & texture parsing code grabbed from it.
* Game uses assets from wolfenstein 3D. Open Sourced by John Carmack https://github.com/id-Software/wolf3d
