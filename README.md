# Minimal FPS Game Gym Environment with Bevy and DQN Agent


https://user-images.githubusercontent.com/97428129/192408641-bb55693e-7716-49b7-9001-d561bda74e37.mp4


**Sergei Surovtsev** <<ssurovsev@gmail.com>>
<br />
February 2022
## Project Description

This project is an attempt to build minimal FPS game with Bevy Game Engine (0.7.0) and create AI agent to play it competitively with human player.

## Project Goals

* Implementing minimal FPS environment with Bevy Game Engine (0.8.0)

## Technical Formulation of Problem

* Implement FPS Gym Environment
* Train an AI agent

## Training Environment

Environment spawns a http server for programmatic interaction. API is available on http://localhost:7878/

Following API handles are exposed:

* **[GET]** `http://localhost:7878/screen.png` — A first-person view of the world by the agent
* **[POST]** `http://localhost:7878/step` — Perform a step and evaluate reward 
    <br />*POST BODY*: ACTION 
    <br />*RESPONSE*: ```json {is_terminated: false, reward: 10.0}```

## Mathematical Models

https://github.com/stillonearth/BevyStein/blob/main/python/DQN.ipynb contains DQN implementation from previous work [1].

## Acknowledgements

* Inspired by [rustenstein](https://github.com/AdRoll/rustenstein) by AdRoll. Map & texture parsing code grabbed from it.
* Game uses assets from wolfenstein 3D. Open Sourced by John Carmack https://github.com/id-Software/wolf3d

## References

1. **Using Deep-Q-Networks (DQN) for solving Unity ML Agents Banana Collector Discreet Control Environment and Evaluating DQN Improvements**, Sergei Surovtsev, July 2019, https://github.com/cwiz/DRLND-Project-Navigation/blob/master/WRITEUP.md
