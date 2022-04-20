# Minimal FPS Game Gym Environment with Bevy and DQN Agent

<img width="578" alt="image" src="https://user-images.githubusercontent.com/97428129/161647148-3140d9a4-f3b1-4237-8f6a-93f31ff48a07.png">

**Sergei Surovtsev** <<ssurovsev@gmail.com>>
<br />
February 2022
## Project Description

This project is an attempt to build minimal FPS game with Bevy Game Engine (0.6.0) and create AI agent to play it competitively with human player.

## Project Goals

* Implementing minimal FPS environment with Bevy Game Engine (0.6.0)

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

## Known Bugs

* Environment wouldn't do proper reset. In order to restart the environment during training you need to shut down the process and run it again. This should be resolved with bevy 0.7 release.

## Acknowledgements

* Inspired by [rustenstein](https://github.com/AdRoll/rustenstein) by AdRoll. Map & texture parsing code grabbed from it.
* Game uses assets from wolfenstein 3D. Open Sourced by John Carmack https://github.com/id-Software/wolf3d

## References

[1] **Using Deep-Q-Networks (DQN) for solving Unity ML Agents Banana Collector Discreet Control Environment and Evaluating DQN Improvements**, Sergei Surovtsev, July 2019, https://github.com/cwiz/DRLND-Project-Navigation/blob/master/WRITEUP.md
