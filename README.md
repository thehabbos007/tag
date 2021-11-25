# Tag!

![screenshot of agents in the environment, one of which is "it"](https://i.imgur.com/NPBcqUL.png)

This project is a simulation of agents playing a game of tag.
The agents are simulated using an [ECS (Shipyard)](https://github.com/leudz/shipyard) and the environment is visualized with [raylib](https://github.com/deltaphc/raylib-rs).

Agents that are colored black are "not it" and agents that are colored gold are "it". When agents that are "it" collide with "not it" agents, tagging happens!

## Running

To make raylib work, it is requried to follow [these instructions](https://github.com/raysan5/raylib/wiki).
The project can then be started by `cargo run --release` from the root directory.
