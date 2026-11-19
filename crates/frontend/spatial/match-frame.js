// Layout emitted by FreightMatch::frame. Keeping the eight actors in the same transfer
// avoids per-actor calls across the WebAssembly boundary on high-refresh screens.
export const field = Object.freeze({
  phase: 0,
  stageTime: 1,
  seconds: 2,
  blue: 3,
  orange: 4,
  x: 5,
  eyeY: 6,
  z: 7,
  yaw: 8,
  pitch: 9,
  health: 10,
  weapon: 11,
  magazine: 12,
  reserve: 13,
  reload: 14,
  damage: 15,
  hit: 16,
  speed: 17,
  crouch: 18,
  respawn: 19,
  shots: 20,
  limit: 21,
  shield: 22,
  cooldown: 23,
  kills: 24,
  deaths: 25,
  elimination: 114,
  round: 115,
  winner: 116
});
export const actor = Object.freeze({
  x: 0,
  y: 1,
  z: 2,
  yaw: 3,
  health: 4,
  speed: 5,
  flash: 6,
  shield: 7,
  kills: 8,
  deaths: 9,
  respawn: 10
});
export const actorOffset = (id) => 26 + id * 11;
