# Minimal FPS Battle Royale Game Environment with Neural Agents using Variant of Deep-Q-Network
Wolfenstein in Bevy

**Sergei Surovtsev** <<ssurovsev@gmail.com>>
<br />
February 2022

https://user-images.githubusercontent.com/97428129/156439447-a288eada-f964-4ec5-a0a2-1653137eb090.mp4

## Project Description

This project is an attempt to build minimal FPS game with Bevy Game Engine (0.6.0) and create AI agent to play it competitively with human player.



### AI Environment

**Goal**

* Kill opposing players
* Survive to the end of round

**Observations**


* Raw pixel input (300x300)

**Actions**

7 discreet actions:

* Step forward, back, left and right
* Turn left and right
* Shoot

** Rewards**

* +1 on player kill
* -1 on death
* +1 on being last surviving player

## Project Goals

* Implementing minimal FPS environment with Bevy Game Engine (0.6.0)
* Researching on feasibility of DQN for competitive FPS games

## Technical Formulation of Problem

* Implement FPS Gym Environment
* Train an AI agent

## Architecture

## Mathematical Models

## Results

## Progress

Currently working

* [+] Collisions
* [+] Shooting
* [+] Map consturction

TODO:

* [ ] Enemies
* [ ] Player health
* [ ] Enemy AI with neural networks

## Acknowledgements

Inspired by [rustenstein](https://github.com/AdRoll/rustenstein) by AdRoll. Map & texture parsing code grabbed from it.

## References

* [1] Sergei Surovtsev, [Using Deep-Q-Networks (DQN) for solving Unity ML Agents Banana Collector Discreet Control Environment and Evaluating DQN Improvements](https://github.com/cwiz/DRLND-Project-Navigation/blob/master/WRITEUP.md), 2019
