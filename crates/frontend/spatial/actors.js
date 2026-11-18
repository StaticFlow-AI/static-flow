import { Group, Mesh } from 'three';
import { Batch } from './mesh.js';

// Geometry follows an eight-pixel-wide head with one texture pixel per modelling unit.
const skins = {
  okabe: { hair: 'ink', coat: 'ivory', shirt: 'latte', pants: 'navy', eyes: 'darkblue' },
  mayuri: { hair: 'darkblue', coat: 'indigo', shirt: 'teal', pants: 'skin', eyes: 'cyan' },
  daru: { hair: 'hair', coat: 'moss', shirt: 'green', pants: 'denim', eyes: 'ink' },
  kurisu: { hair: 'auburn', coat: 'khaki', shirt: 'ivory', pants: 'ink', eyes: 'violet' },
  moeka: { hair: 'chestnut', coat: 'purple', shirt: 'grey', pants: 'navy', eyes: 'violet' },
  luka: { hair: 'black', coat: 'white', shirt: 'white', pants: 'purple', eyes: 'violet' },
  faris: { hair: 'rose', coat: 'ink', shirt: 'white', pants: 'black', eyes: 'orange' },
  suzuha: { hair: 'chestnut', coat: 'moss', shirt: 'cream', pants: 'denim', eyes: 'green' }
};
function part(material, draw) {
  const b = new Batch();
  draw((x, y, z, w, h, d, color) => b.box(x / 16, y / 16, z / 16, w / 16, h / 16, d / 16, color));
  return new Mesh(b.geometry(), material);
}
export function character(actor, material) {
  const id = actor.id,
    s = skins[id],
    root = new Group();
  root.position.set(...actor.pos);
  root.rotation.y = actor.angle;
  const stout = id === 'daru' ? 1 : 0;
  const body = part(material, (b) => {
    b(-3.4 - stout, 10, -1.8, 6.8 + stout * 2, 8.2, 3.7, s.coat);
    b(-2.1, 10.1, 1.9, 4.2, 7.7, 0.25, s.shirt);
    b(-3.65, 1.1, -1.6, 3.25, 9, 3.1, s.pants);
    b(0.4, 1.1, -1.6, 3.25, 9, 3.1, s.pants);
    for (const x of [-3.65, 0.4]) {
      b(x, 0, -1.7, 3.25, 1.6, 4, 'deepwood');
      b(x + 0.2, 1.2, 1.2, 2.85, 0.5, 0.9, 'latte');
    }
    b(-1.4, 17.4, -1, 2.8, 1.9, 2.2, 'skin');
    if (id === 'okabe') {
      for (const x of [-3.6, 1.5]) {
        b(x, 7.5, -1.8, 2.1, 8.4, 4, 'ivory');
        b(x, 11, 2.2, 2.1, 1, 0.3, 'latte');
      }
      b(-2.2, 15.4, 2.2, 1, 2.8, 0.3, 'white');
      b(1.1, 15.4, 2.2, 1, 2.8, 0.3, 'white');
    }
    if (id === 'kurisu') {
      b(-0.6, 12, 2.3, 1.3, 5.4, 0.3, 'red');
      b(-3.6, 9.4, -1.8, 7.3, 1, 4, 'deepwood');
      b(-3.5, 10.4, 1.9, 7, 0.8, 0.4, 'cream');
    }
    if (id === 'mayuri' || id === 'luka' || id === 'faris') {
      b(-4.1, 7.1, -2.1, 8.2, 4.1, 4.7, id === 'mayuri' ? 'indigo' : s.pants);
      if (id === 'faris') {
        b(-2.2, 8.1, 2.65, 4.4, 3, 0.3, 'white');
        b(-2.5, 14, 2.25, 5, 2, 0.3, 'paper');
      }
    }
    if (id === 'daru') {
      b(-4.2, 10.2, 1.5, 8.4, 4.8, 1.2, 'moss');
      b(-2, 14, 2.11, 3, 2.4, 0.3, 'cream');
    }
    if (id === 'suzuha') {
      b(-3.4, 13.9, 1.92, 6.8, 0.65, 0.25, 'cream');
      b(-3.2, 10, 1.92, 6.4, 0.8, 0.3, 'deepwood');
    }
  });
  root.add(body);
  const head = part(material, (b) => {
    b(-3.5, 0, -3, 7, 7.1, 6, 'skin');
    b(-3.7, 5.3, -3.2, 7.4, 2.4, 6.5, s.hair);
    b(-3.65, 0.1, -3.2, 7.3, 6, 1.7, s.hair);
    b(-3.6, 2.6, -2.3, 1, 3.6, 5.3, s.hair);
    b(2.6, 2.5, -2.3, 1, 3.7, 5.3, s.hair);
    b(-3.5, 5, 2.94, 6.9, 1.3, 0.4, s.hair);
    b(-2.1, 4.2, 3, 1.5, 1.4, 0.45, s.hair);
    for (const x of [-2.65, 0.7]) {
      b(x, 2.7, 3.02, 2, 0.95, 0.15, 'paper');
      b(x + 0.6, 2.65, 3.2, 1, 1.15, 0.15, s.eyes);
      b(x + 0.15, 3.85, 3.1, 1.7, 0.36, 0.2, s.hair);
    }
    b(-0.65, 1.05, 3, 0.99, 0.32, 0.2, 'darkskin');
    if (['kurisu', 'moeka', 'faris', 'suzuha'].includes(id)) {
      for (const x of [-3.65, 2.55]) b(x, -2.1, -2.8, 1.15, 5.6, 3.7, s.hair);
      b(-2.7, -2.4, -3, 5.4, 6, 1.4, s.hair);
    }
    if (id === 'okabe') {
      b(-3.2, 7, -2.9, 2.5, 1.4, 3, s.hair);
      b(0.3, 7.4, -2.8, 2, 1.1, 2.2, s.hair);
      b(1.8, 4.2, 3, 1.4, 1.8, 0.5, s.hair);
    }
    if (id === 'mayuri') {
      b(-4.4, 6.15, -3.9, 8.8, 1.25, 7.7, 'blue');
      b(-3.6, 7.3, -3.2, 7.2, 1.4, 6.6, 'indigo');
      b(-1.7, 8.4, -2.2, 3.8, 0.65, 3.7, 'blue');
    }
    if (id === 'daru') {
      b(-3.8, 6.1, -3.75, 7.6, 1.5, 7, 'gold');
      b(-3.5, 7.4, -2.8, 7, 1.2, 5, 'orange');
      b(-3.4, 6, 2.7, 6.8, 0.7, 2.1, 'gold');
    }
    if (id === 'daru' || id === 'moeka') {
      for (const x of [-3.2, 0.45]) {
        b(x, 3.85, 3.35, 2.8, 0.4, 0.28, 'ink');
        b(x, 2.35, 3.35, 2.8, 0.4, 0.28, 'ink');
        b(x, 2.7, 3.35, 0.4, 1.25, 0.28, 'ink');
        b(x + 2.4, 2.7, 3.35, 0.4, 1.25, 0.28, 'ink');
      }
      b(-0.5, 3.3, 3.38, 1.1, 0.35, 0.2, 'ink');
    }
    if (id === 'faris') {
      for (const x of [-4.9, 3.1]) {
        b(x, 3.7, -2, 2, 2.5, 2.7, 'pink');
        b(x, -1, -2.5, 2.1, 5.2, 2.6, 'rose');
        b(x + 0.5, 7, -1, 1.6, 1.7, 1.7, 'white');
      }
      b(-2.4, 7.15, -2, 4.8, 0.95, 3, 'paper');
    }
    if (id === 'suzuha') {
      b(2.7, 0.6, -3.8, 2, 3.3, 2.5, 'chestnut');
      b(2.9, -2.5, -3.9, 1.6, 3.1, 2, 'hair');
      b(2.9, -0.1, -3.9, 1.85, 0.7, 2.2, 'gold');
    }
    if (id === 'luka') {
      b(-3.8, 3.8, 1, 1, 2, 2.2, 'rose');
    }
  });
  head.position.y = 18.2 / 16;
  root.add(head);
  const arms = [];
  for (const side of [-1, 1]) {
    const arm = part(material, (b) => {
      b(-1.25, -5.2, -1.5, 2.5, 5.2, 3.1, s.coat);
      b(-1.15, -8, -1.35, 2.3, 3, 2.8, 'skin');
    });
    arm.position.set((side * (4.4 + stout)) / 16, 17.6 / 16, 0);
    arm.rotation.z = side * 0.06;
    arm.rotation.x = side === 1 && id === 'moeka' ? -0.85 : -0.09;
    root.add(arm);
    arms.push(arm);
  }
  const size = id === 'daru' ? 1.09 : id === 'mayuri' ? 0.99 : 1.06;
  root.scale.setScalar(size);
  return { root, head, arms };
}
export function hand(material) {
  const block = part(material, (b) => {
    b(-1.7, -6, -1.8, 3.4, 6.2, 3.6, 'ivory');
    b(-1.55, -0.3, -1.6, 3.1, 3.4, 3.25, 'skin');
    b(-1.7, -1.1, -1.8, 3.4, 0.9, 3.6, 'latte');
  });
  return block;
}
