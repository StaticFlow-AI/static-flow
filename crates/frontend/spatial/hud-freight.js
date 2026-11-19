import { field, actor, actorOffset } from './match-frame.js';
import { cargo } from './ship.js';

export const names = ['你', '回声', '灰鲨', '舷灯', '赤隼', '海燕', '砂岩', '北风'];
const timeText = (s) => Math.floor(Math.ceil(s) / 60) + ':' + String(Math.ceil(s) % 60).padStart(2, '0');
const escButton = (act, label) => '<button type="button" data-act="' + act + '">' + label + '</button>';
const clock = timeText;
export function hud(host, actions, signal, credits) {
  const root = document.createElement('div');
  root.className = 'deck-hud';
  host.append(root);
  root.innerHTML = `
    <div class="deck-radar"><canvas width="186" height="186" aria-label="甲板小地图"></canvas><span>运输船</span></div>
    <div class="deck-score" aria-label="队伍比分"><b class="port">0</b><div><small>团队竞赛</small><strong class="time">4:00</strong></div><b class="star">0</b></div>
    <div class="deck-feed" aria-live="polite"></div><div class="deck-flare"></div><div class="deck-sight"><i></i><i></i><i></i><i></i></div><div class="deck-hit"></div>
    <div class="deck-life"><span class="med">✚</span><b>100</b><span class="life-bar"><i></i></span></div>
    <div class="deck-ammo"><span class="gun-name">AR-31</span><strong>30</strong><small>/ 120</small><i></i></div>
    <div class="deck-message" role="status"></div>
    <div class="deck-menu"><section><h1>运输船</h1><div class="deck-rule"><button data-act="rules-race" aria-pressed="true">团队竞赛</button><button data-act="rules-rounds" aria-pressed="false">回合淘汰</button></div><div class="deck-objective">4 对 4　/　率先取得 20 分</div>
      <div class="deck-actions">${escButton('start', '空降加入　↗')}${escButton('settings', '操作与画质')}</div>
    </section></div>
    <div class="deck-dialog" hidden></div>
    <div class="deck-board" hidden></div>
    <div class="deck-touch"><div class="deck-stick"><i></i></div><div class="deck-touch-actions">
      <button data-touch="aim">瞄准</button><button data-touch="jump">跳</button><button data-touch="reload">换弹</button><button data-touch="switch">切枪</button><button data-touch="fire">开火</button></div><button class="touch-pause" data-touch="pause" aria-label="暂停">Ⅱ</button></div>`;
  const get = (s) => root.querySelector(s),
    radar = get('.deck-radar canvas'),
    ctx = radar.getContext('2d');
  const map = document.createElement('canvas');
  map.width = 186;
  map.height = 186;
  const c = map.getContext('2d');
  const scale = 2.62,
    X = (x) => 93 + x * scale,
    Y = (z) => 93 + z * scale;
  c.fillStyle = '#0d191dd9';
  c.fillRect(0, 0, 186, 186);
  c.strokeStyle = '#7b8e8c28';
  c.lineWidth = 1;
  for (let x = 5; x < 186; x += 21) {
    c.beginPath();
    c.moveTo(x, 0);
    c.lineTo(x, 186);
    c.moveTo(0, x);
    c.lineTo(186, x);
    c.stroke();
  }
  c.beginPath();
  for (const [x, z] of [
    [-15, 28],
    [-15, -26],
    [-11, -32],
    [0, -35],
    [11, -32],
    [15, -26],
    [15, 28],
    [12, 31],
    [-12, 31]
  ])
    c.lineTo(X(x), Y(z));
  c.closePath();
  c.fillStyle = '#4c5755';
  c.fill();
  c.strokeStyle = '#97a29b';
  c.stroke();
  for (const o of cargo) {
    c.fillStyle = o.kind === 'container' ? '#a5ada2' : o.kind === 'crate' ? '#868d81' : '#697d79';
    c.fillRect(X(o.x - o.w / 2), Y(o.z - o.d / 2), o.w * scale, o.d * scale);
  }
  const score = [get('.port'), get('.star')],
    time = get('.time'),
    matchLabel = get('.deck-score small'),
    life = get('.deck-life b'),
    bar = get('.life-bar i'),
    clip = get('.deck-ammo strong'),
    reserve = get('.deck-ammo small'),
    weapon = get('.gun-name'),
    ammoBar = get('.deck-ammo i'),
    sight = get('.deck-sight'),
    hit = get('.deck-hit'),
    dmg = get('.deck-flare'),
    msg = get('.deck-message'),
    menu = get('.deck-menu'),
    dialog = get('.deck-dialog'),
    board = get('.deck-board'),
    feed = get('.deck-feed');
  let mode = '',
    panel = '',
    lastPhase,
    frame,
    t = 0,
    lastRender = -1,
    lastBoard = '',
    feedItems = [],
    touch = matchMedia('(pointer:coarse)').matches || navigator.maxTouchPoints > 0;
  if (touch) root.classList.add('handheld');
  function text(el, value) {
    const s = String(value);
    if (el.textContent !== s) el.textContent = s;
  }
  function controls() {
    const rows = touch
      ? [
          ['左侧摇杆', '移动'],
          ['滑动右半屏', '环顾'],
          ['右下按钮', '开火、瞄准、跳跃、换弹']
        ]
      : [
          ['W A S D / Shift', '移动 / 冲刺'],
          ['鼠标 / 右键', '环顾 / 瞄准'],
          ['左键 / R', '开火 / 换弹'],
          ['1、2 / 滚轮', '切换主副武器'],
          ['空格 / C', '跳跃 / 蹲下'],
          ['Tab / Esc', '计分板 / 暂停']
        ];
    return rows.map(([a, b]) => '<li><kbd>' + a + '</kbd><span>' + b + '</span></li>').join('');
  }
  function open(which) {
    panel = which;
    dialog.hidden = !which;
    if (!which) {
      root.classList.remove('modal-up');
      return;
    }
    root.classList.add('modal-up');
    if (which === 'pause')
      dialog.innerHTML =
        '<section><h2>比赛暂停</h2><div class="deck-actions">' +
        escButton('resume', '继续比赛') +
        escButton('settings', '操作与画质') +
        '</div></section>';
    else if (which === 'settings') {
      const prefs = actions.prefs();
      dialog.innerHTML =
        '<section><h2>操作与画质</h2><ul>' +
        controls() +
        '</ul><div class="deck-choices" aria-label="画质">' +
        [
          ['auto', '自动'],
          ['native', '原生精细'],
          ['fast', '流畅']
        ]
          .map(
            ([k, n]) =>
              '<button data-act="quality-' +
              k +
              '" aria-pressed="' +
              (prefs.quality === k) +
              '">' +
              n +
              '</button>'
          )
          .join('') +
        '</div><div class="deck-actions">' +
        escButton('sound', prefs.mute ? '声音 · 关' : '声音 · 开') +
        escButton('back', '返回') +
        '</div><a class="deck-credit" target="_blank" rel="noopener" href="' +
        credits.href +
        '">素材署名</a></section>';
    }
  }
  function setBoard(visible) {
    if (!frame || !visible) {
      board.hidden = true;
      lastBoard = '';
      return;
    }
    const end = frame[field.phase] === 4,
      rows = [];
    for (let i = 0; i < 8; i++) {
      const n = actorOffset(i);
      rows.push([i, frame[n + actor.kills], frame[n + actor.deaths]]);
    }
    const key = [end, frame[field.blue], frame[field.orange], ...rows.flat()].join('/');
    if (key !== lastBoard) {
      const side = (s) =>
        '<div><h3>' +
        ['蓝队', '橙队'][s] +
        '<b>' +
        frame[field.blue + s] +
        '</b></h3><div class="deck-row deck-columns"><span>队员</span><span>击败</span><span>倒地</span></div>' +
        rows
          .filter(([id]) => Number(id >= 4) === s)
          .sort((a, b) => b[1] - a[1] || a[2] - b[2])
          .map(
            ([id, k, d]) =>
              '<div class="deck-row"><strong>' +
              names[id] +
              '</strong><span>' +
              k +
              '</span><span>' +
              d +
              '</span></div>'
          )
          .join('') +
        '</div>';
      board.innerHTML =
        '<section><h2>' +
        (end
          ? frame[field.blue] === frame[field.orange]
            ? '双方平局'
            : frame[field.blue] > frame[field.orange]
              ? '蓝队胜利'
              : '橙队胜利'
          : '甲板计分板') +
        '</h2><div class="deck-teams">' +
        side(0) +
        side(1) +
        '</div>' +
        (end ? escButton('restart', '再来一场　↗') : '') +
        '</section>';
      lastBoard = key;
    }
    board.hidden = false;
  }
  function update(f, dt, now, held) {
    frame = f;
    t = now;
    const phase = f[field.phase];
    if (phase !== lastPhase) {
      mode = ['ready', 'drop', 'live', 'down', 'over', 'interval'][phase];
      root.dataset.phase = mode;
      menu.hidden = phase !== 0;
      if (phase === 0 || phase === 4) open('');
      if (phase === 4) setBoard(true);
      lastPhase = phase;
    }
    text(matchLabel, f[field.elimination] ? '回合 ' + f[field.round] : '团队竞赛');
    text(score[0], f[field.blue]);
    text(score[1], f[field.orange]);
    text(time, clock(f[field.seconds]));
    text(life, f[field.health]);
    text(clip, f[field.magazine]);
    text(reserve, '/ ' + f[field.reserve]);
    text(weapon, f[field.weapon] ? 'P-12  /  副武器' : 'AR-31  /  主武器');
    bar.style.transform = 'scaleX(' + f[field.health] / 100 + ')';
    ammoBar.style.transform = 'scaleX(' + f[field.magazine] / (f[field.weapon] ? 12 : 30) + ')';
    const progress = f[field.reload];
    text(
      msg,
      phase === 1
        ? '正在空降　' + Math.max(1, Math.ceil(3.1 - f[field.stageTime]))
        : phase === 5
          ? (f[field.winner] < 0 ? '回合平局' : f[field.winner] === 0 ? '蓝队赢得回合' : '橙队赢得回合') +
            '　·　交换甲板'
          : phase === 3
            ? f[field.elimination]
              ? '队友观战'
              : '重新部署　' + Math.max(1, Math.ceil(f[field.respawn]))
            : progress >= 0
              ? '更换弹匣'
              : phase === 2 && f[field.shield] > 0.15
                ? '部署保护'
                : ''
    );
    hit.style.opacity = f[field.hit];
    dmg.style.opacity = f[field.damage] * 0.8;
    sight.style.setProperty(
      '--gap',
      (held.aim ? 3 : 5 + f[field.speed] * 0.53 + (f[field.cooldown] > 0.06 ? 2 : 0)) + 'px'
    );
    if (now - lastRender > 0.085) {
      ctx.clearRect(0, 0, 186, 186);
      ctx.drawImage(map, 0, 0);
      for (let id = 7; id >= 0; id--) {
        const i = actorOffset(id);
        if (!f[i + actor.health]) continue;
        if (
          id >= 4 &&
          f[i + actor.flash] <= 0 &&
          Math.hypot(f[i] - f[field.x], f[i + actor.z] - f[field.z]) > 12
        )
          continue;
        ctx.save();
        ctx.translate(X(f[i]), Y(f[i + actor.z]));
        ctx.rotate(-(id ? f[i + actor.yaw] : f[field.yaw]));
        ctx.fillStyle = id === 0 ? '#ffda75' : id < 4 ? '#70c9fd' : '#ff9470';
        ctx.beginPath();
        if (id === 0) {
          ctx.moveTo(0, -6);
          ctx.lineTo(4.1, 4.5);
          ctx.lineTo(0, 2.6);
          ctx.lineTo(-4.1, 4.5);
        } else ctx.arc(0, 0, 2.9, 0, Math.PI * 2);
        ctx.closePath();
        ctx.fill();
        ctx.restore();
      }
      lastRender = now;
      feedItems = feedItems.filter((item) => {
        if (now < item.until) return true;
        item.el.remove();
        return false;
      });
    }
    if (phase === 4 || held.board) setBoard(true);
    else if (!board.hidden) setBoard(false);
  }
  function elimination(id, target) {
    const el = document.createElement('div');
    el.className = id < 4 ? 'blue-kill' : 'orange-kill';
    el.innerHTML = '<b>' + names[id] + '</b><span> › </span><em>' + names[target] + '</em>';
    feed.prepend(el);
    feedItems.push({ el, until: t + 4.5 });
    if (feedItems.length > 4) feedItems.shift().el.remove();
  }
  root.addEventListener(
    'click',
    (e) => {
      const act = e.target.closest('[data-act]')?.dataset.act;
      if (!act) return;
      if (act.startsWith('rules-')) {
        const rounds = act === 'rules-rounds';
        actions.rules(rounds);
        for (const btn of root.querySelectorAll('[data-act^="rules-"]'))
          btn.setAttribute('aria-pressed', String(btn.dataset.act === act));
        text(
          get('.deck-objective'),
          rounds ? '4 对 4　/　每回合一条命，先胜三回合' : '4 对 4　/　率先取得 20 分'
        );
      } else if (act === 'settings') {
        actions.pause(false);
        open('settings');
      } else if (act === 'back') {
        if (frame && frame[field.phase] !== 0) open('pause');
        else open('');
      } else if (act.startsWith('quality-')) {
        actions.quality(act.slice(8));
        open('settings');
      } else if (act === 'sound') {
        actions.sound();
        open('settings');
      } else if (act === 'start' || act === 'restart') {
        open('');
        for (const { el } of feedItems) el.remove();
        feedItems = [];
        actions.start();
      } else if (act === 'resume') {
        open('');
        actions.resume();
      }
    },
    { signal }
  );
  return { root, update, open, panel: () => panel, elimination, touch };
}
