## Next

- Shots
- Friendly Units
- Shrapnel
- Saucers

## Future

- Modes
  - Attract Mode (Intro)
  - Game Levels (of increasing difficulty)
  - Level Complete / Interstitial (shows player score at end of each level)
- Modding
  - Modding is an important aspect of what I would like to do: ideally we want each type of enemy
    ship to be it's own mod. However, we will need to figure out how to integrate the modded
    additions into the level progression
  - Mods export a number of Challenges.
- Level Progression
  - the goal is to get as high as possible in the progression (Rogue-like, but with lives)
  - A level consists of a set of challenges
  - A challenge can spawn a number of enemy ships, either randomly or in formation.
  - A level ends when all challenges have been overcome.
  - Each challenge has a difficulty rating, as well as a minimum level.
  - Each level, we choose a random set of challenges such that the total difficulty of all
    challenges is proportional to the current progression level.
  - This means that as you progress, you will see more enemies, or new enemy types
  - some enemies spawn immediately upon entering a level, other types show up later.
  - this means that if a mod exports a challenge whose difficulty is overrated, adding that mod may
    cause the game to get easier, as formations from that challenge will displace other enemies.
- Rescues
  - Challenges may also spawn friendly objects, which can be abducted by saucers.
  - if you kill the saucer abducting a friendly, you'll need to rescue the friendly before it
    falls to the ground and explodes
  - rescuing a friendly grants mega-points and possibly lives
- Power ups
  - extra beams
  - shields
  - smart bombs
- Enemy types
  - Saucers
  - Mines
  - Seeker Drones
  - Turrets
  - Interceptors
  - Others to be added later
- Camera movement
  - viewpoint
