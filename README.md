# Tag!

This project is a simulation of agents (players) playing a game of tag.
The agents are simulated using an [ECS (Shipyard)](https://github.com/leudz/shipyard) and the environment is visualized with [raylib](https://github.com/deltaphc/raylib-rs).

---

- Agents that are colored black are "not it"
- Agents that are colored gold are "it"
- When agents that are "it" collide with "not it" agents, tagging happens!
- Every player has endurance which depletes over time, and regenerates in pieces over time as it's used
  - Tagged players regenerate endurance faster than untagged players
- Untagged players can wrap edges of the playing area
- Tagged players cannot wrap corners of the playing area

Fairness between the endurance regeneration is balanced by the tagged players being able to wrap the screen in order to get to safety.

![screenshot of agents in the environment, one of which is "it"](https://i.imgur.com/L1gNfh5.png)

## Running simulations

To make raylib work, it is requried to follow [these instructions](https://github.com/raysan5/raylib/wiki).
The project can then be started by `cargo run --release -- 100` from the root directory. This starts a simulation with 100 players.

## Architecture

The simulations run on an [ECS (Shipyard)](https://github.com/leudz/shipyard) where the main frame of mind is structure-of-arrays.
What this means, is that rahter than having data objects as structures in a vector (which is a common way to handle cohesive data) data is sliced
and grouped by their data type. This creates a structure of arrays, and the operations in the program happens across the arrays.

`Entities` are a conceptual unit that has many `Components`. We can think of an index into the arrys as being the identifier for the `Entity`.
Every `Component` with the same index forms an `Entity`. The operations in the program are `Systems`, which act on `Entities`/`Components` in iterations.

![screenshot of struct of arrays](https://i.imgur.com/p3IX4PA.png)

In order to progress the state of the world, the `Systems` are run in `TICKS` which little-by-little apply rules to the data in bulk.
That's how the simulation is built. All this in combination can be denoted as the `World`.

![systems in program](https://i.imgur.com/AS12e73.png)

Visuzlization of the `World` can happen with any graphics library that can take the `Entities` and show meaningful graphics form their `Components`.
In this case position and direction/speed of each player. In the case of this project, [raylib](https://github.com/deltaphc/raylib-rs) is used for grapics.

Each player has its own behaviour. Behaviours can be customized using different behaviour implementations. See `src/behaviours`. Using some context about the environment
and the sorrounding players, we can change the direction of the agent.

## Performance

Initial benchmarking of performance (and flamegraphs) showed lots of time spent on finding closest neighbours and colliding.
These operations could be around `O(n^2)` execution time as each player needs to traverse every other player.
Switching to using a R\*-Tree ( implemented in [spade](https://github.com/Stoeoef/spade)) made running collisions (and finding nearest neighbours) a great bit faster.
Benchmarking the environment with 1000 players. The following numbers are seconds used per tick.

```
# Manual iteration
tick world              time:   [2.9338 ms 2.9406 ms 2.9478 ms]
                        change: [-0.3157% -0.0490% +0.2270%] (p = 0.73 > 0.05)

# R*-Tree collision and nearest neighbors
tick world              time:   [1.5481 ms 1.5512 ms 1.5543 ms]
                        change: [-47.318% -47.163% -47.003%] (p = 0.00 < 0.05)
                        Performance has improved.
```

[Shipyard](https://github.com/leudz/shipyard) comes with a fair bit of performance tweaks out of the box. For example workloads which orchestrate systems are run in parallel using Rayon when possible.
Furhtermore the storage used for Shipyard is very iteration-friendly, which means it's well-optimized for having a large amount of entities.

Without rendering, the simulation can handle up to 16000 agents _in real-time_ assuming a target of 33 ms/tick (30 ticks per second).

```
tick world              time:   [32.956 ms 33.014 ms 33.073 ms]
                        change: [+0.9331% +1.1934% +1.4402%] (p = 0.00 < 0.05)

```

And the simulation engine can handle millions of agents if real-time is not a concert, but at around 3 seconds/tick.

There is some potential for further parallelism through parallel iteration in Rayon, but the current setup does not allow for that trivially. For basic sing-component iterations, it is possible, but for iterations across entities with multiple components, the plumbing needed is not in place out-of-the box.
