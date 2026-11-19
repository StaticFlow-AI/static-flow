const actions = {
  phone: '接听手机',
  crt: '接通显像管',
  wave: '操作微波炉',
  leap: '检查耳机',
  upa: '调查乌帕',
  nixie: '读取数值',
  bench: '使用工作台',
  furnace: '使用熔炉'
};
const help = [
  ['W A S D / 方向键上下', '移动，Shift 奔跑，空格跳跃。左右方向键也能转身，Page Up / Down 抬头低头。'],
  ['鼠标左键 / F', '按住开采准星所指的方块。矿石需要相应等级的镐。'],
  ['鼠标右键 / R', '把手持方块放在准星所指的表面。数字键 1–9 或滚轮切换快捷栏。'],
  ['E / B / M', '调查人物与设备，打开背包，或查看随身手机。Esc 关闭面板。'],
  ['V · 自由俯瞰', '切换自由飞行或回到角色。空格上升，C 下降，Shift 加速。'],
  ['触屏', '左下摇杆移动，划动场景转头，用右侧按钮跳跃、调查、采集和放置。']
];
const climates = { clear: '晴', rain: '雨', thunder: '雷暴' };
const guide = [
  [
    '取水与引流',
    '西南街口的石井（X −11，Z 14）有水源。拿起背包里的空木桶（新世界在第九格），右键或触屏「用」取水，再对准地面倒水。挖沟可以引流，移走水源后流水会退去。'
  ],
  [
    '村民与夜行生物',
    '沿研究所外的街道找村民，走近按 E 查价并交换。夜里和雷暴中留意僵尸，按住采集键还击。雨天村民会往家里走。'
  ],
  [
    '自由俯瞰',
    '点击「自由俯瞰」或按 V 即可离开角色观察。WASD 平移，空格升高，C 降低，Shift 加速。按 G 或点击「地图与天气」可选地标、时刻与天气，返回生存会回到原地。'
  ],
  [
    '从研究所出发',
    '在二楼沙发前的茶几拿手机。二楼东侧有两道外楼梯，靠墙的一道下到街上，外侧的一道通往屋顶。'
  ],
  [
    '木器与石器',
    '街南的树可以徒手采原木。用背包做木板和木棍，在工作台合成木镐。二楼西墙有现成工作台，也能自己做一个随身带走。'
  ],
  [
    '露天矿与冶炼',
    '沿街往东，过斑马线就是露天矿（X 24，Z −3）。木镐采石和煤，石镐采铜，铜镐采蓝绿色晶簇。六块圆石可合成熔炉，放好后靠近它，用煤熔铜。'
  ],
  [
    '修复电话微波炉',
    '在工作台用两枚铜锭和一枚晶体合成中继器，装在二楼微波炉相邻的一格。去楼下开显像管，回来拨动炉盘，再用手机发 D-Mail。'
  ],
  ['改变过去', '换线之后，调查留下来的人和电脑桌上的耳机。把记忆带回研究所，寻找能让两个人一起留下的线索。'],
  [
    '野外与建造',
    '泥土上种下橡树苗，留足上方空间，一段时间后就能再砍。采橡树叶也能找到果实。高处坠落会受伤，落水可以缓冲。团聚后可继续探索、烧玻璃、做灯、修房子。'
  ]
];
const node = (tag, cls, text) => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text) e.textContent = text;
  return e;
};
function btn(text, action, id) {
  const b = node('button', '', text);
  b.type = 'button';
  b.dataset.action = action;
  if (id !== undefined) b.dataset.id = id;
  return b;
}
export function makeHud(host, cat, itemIcon, handle, signal, graphics, mobHint) {
  const root = node('div', 'voxel-hud');
  host.append(root);
  root.innerHTML =
    '<div class="voxel-place"><strong></strong><span></span></div>' +
    '<div class="voxel-clock"><small>世界线变动率</small><b></b></div>' +
    '<div class="voxel-cross" aria-hidden="true"></div><div class="voxel-focus"></div><div class="voxel-break"><i></i></div>' +
    '<div class="voxel-bottom"><div class="voxel-storage" role="status"></div><div class="voxel-message" role="status"></div><div class="voxel-vitals"></div><div class="voxel-bar" role="group" aria-label="物品快捷栏"></div><div class="voxel-hint"></div></div>' +
    '<div class="voxel-menu"></div><div class="voxel-stick" aria-label="移动摇杆"><i></i></div><div class="voxel-touch"></div>';
  const $ = (s) => root.querySelector(s);
  const place = $('.voxel-place strong'),
    coords = $('.voxel-place span'),
    clock = $('.voxel-clock b'),
    bar = $('.voxel-bar');
  const focus = $('.voxel-focus'),
    notice = $('.voxel-message'),
    progress = $('.voxel-break i'),
    health = $('.voxel-vitals');
  const hint = $('.voxel-hint'),
    menu = $('.voxel-menu'),
    touch = $('.voxel-touch');
  const coarse = matchMedia('(pointer:coarse)').matches || navigator.maxTouchPoints > 0;
  if (coarse) root.classList.add('voxel-coarse');
  for (const [label, key] of [
    ['自由俯瞰', 'survey'],
    ['地图与天气', 'sky'],
    ['背包', 'bag'],
    ['手册', 'journal'],
    ['手机', 'phone']
  ])
    menu.append(btn(label, key));
  const touchButtons = [
    ['升', 'rise', '按住上升'],
    ['降', 'descend', '按住下降'],
    ['跳', 'jump', '跳跃'],
    ['用', 'use', '调查 / 使用'],
    ['采', 'mine', '按住采集'],
    ['放', 'place', '放置方块']
  ];
  for (const [label, key, aria] of touchButtons) {
    const b = btn(label, key);
    b.setAttribute('aria-label', aria);
    touch.append(b);
  }
  const skyReadout = node('span', 'voxel-weather');
  $('.voxel-place').append(skyReadout);
  const surveyButton = menu.querySelector('[data-action="survey"]');
  const slots = Array.from({ length: 9 }, (_, i) => {
    const b = btn('', 'slot', i);
    bar.append(b);
    return b;
  });
  const dialog = node('dialog', 'voxel-dialog');
  host.append(dialog);
  const wrap = node('div', 'voxel-sheet');
  dialog.append(wrap);
  const heading = node('div', 'voxel-sheet-head'),
    title = node('h1');
  heading.append(title, btn('关闭 ×', 'close'));
  wrap.append(heading);
  const content = node('div', 'voxel-sheet-content');
  wrap.append(content);
  const picture = (id) => {
    const img = node('img');
    img.src = itemIcon(id);
    img.alt = '';
    img.draggable = false;
    return img;
  };
  let mode = '',
    s,
    focusKey = '',
    previousProgress = -1,
    slotKeys = [],
    lastHealth = -1,
    timeout,
    placeKey = '';
  function toast(message) {
    clearTimeout(timeout);
    notice.textContent = message || '';
    if (message)
      timeout = setTimeout(() => {
        notice.textContent = '';
      }, 2800);
  }
  function section(parent, caption) {
    const h = node('h2', '', caption);
    parent.append(h);
  }
  function paragraph(parent, text) {
    parent.append(node('p', '', text));
  }
  function addButton(parent, text, key, id, disabled = false) {
    const b = btn(text, key, id);
    b.disabled = disabled;
    parent.append(b);
    return b;
  }
  function inventory() {
    const grid = node('div', 'voxel-pack');
    section(content, '随身物品');
    content.append(grid);
    let count = 0;
    s.bag.forEach((n, id) => {
      if (!n) return;
      count++;
      const b = btn('', 'equip', id);
      b.setAttribute('aria-label', '拿起' + cat.items[id].name);
      b.title = '拿起' + cat.items[id].name;
      if (s.bar[s.slot] === id) b.classList.add('selected');
      b.append(picture(id), node('span', '', cat.items[id].name), node('b', '', String(n)));
      grid.append(b);
    });
    if (!count) paragraph(grid, '背包是空的。按住采集带走第一块木头，或拆下研究所里的一块木板。');
    section(content, '合成与烧制');
    const stations = node(
      'p',
      'voxel-stations',
      s.bench && s.furnace
        ? '附近有工作台和熔炉。'
        : s.bench
          ? '工作台在手边。'
          : s.furnace
            ? '熔炉在手边。'
            : '手工合成可用。靠近工作台或熔炉可以制作更多物品。'
    );
    content.append(stations);
    const recipes = node('div', 'voxel-recipes');
    content.append(recipes);
    const stationName = (id) => (id ? cat.items[id].name : '徒手');
    for (const r of cat.recipes) {
      const available =
        (!r.station || (cat.items[r.station].key === 'bench' ? s.bench : s.furnace)) &&
        r.ingredients.every(([id, q]) => s.bag[id] >= q);
      const b = btn('', 'craft', r.output);
      b.disabled = !available;
      b.setAttribute('aria-label', '制作' + cat.items[r.output].name);
      const info = node('span');
      info.append(
        node('strong', '', cat.items[r.output].name + (r.amount > 1 ? ' ×' + r.amount : '')),
        node(
          'small',
          '',
          stationName(r.station) +
            ' · ' +
            r.ingredients.map(([id, n]) => cat.items[id].name + ' ' + s.bag[id] + '/' + n).join('、')
        )
      );
      b.append(picture(r.output), info);
      recipes.append(b);
    }
  }
  function panel(next, keepScroll = false) {
    const old = mode,
      scroll = content.scrollTop;
    mode = next;
    root.classList.toggle('voxel-paused', !!mode);
    if (!mode) {
      if (dialog.open) dialog.close();
      return;
    }
    if (!dialog.open) dialog.showModal();
    const names = {
      bag: '背包与工作台',
      journal: '研究所手册',
      phone: '翻盖手机　' + s.clock,
      talk: s.panel?.who || '对话',
      leap: '时间跳跃装置',
      help: '操作',
      sky: '地图与天气',
      trade: s.panel?.who || '村民交换',
      reset: '重新生成世界'
    };
    title.textContent = names[mode];
    dialog.dataset.panel = mode;
    content.replaceChildren();
    if (mode === 'bag') inventory();
    if (mode === 'trade' && s.panel?.type === 'trade') {
      const { price, units, reward, count, stock } = s.panel;
      const row = node('div', 'voxel-offer');
      row.append(
        picture(price),
        node('span', '', cat.items[price].name + ' × ' + units),
        node('i', '', '→'),
        picture(reward),
        node('span', '', cat.items[reward].name + ' × ' + count)
      );
      content.append(row);
      paragraph(
        content,
        '持有' +
          cat.items[price].name +
          ' × ' +
          s.bag[price] +
          '　·　' +
          (stock ? '村民存货 ' + stock + ' 份' : '这批货已换完，稍后会补货')
      );
      addButton(content, '交换', 'exchange', undefined, s.bag[price] < units || !stock);
    }
    if (mode === 'sky') {
      const choices = () => {
        const group = node('div', 'voxel-links');
        content.append(group);
        return group;
      };
      section(content, '飞往地标');
      const landmarks = choices();
      ['研究所', '街区村落', '村口水井', '露天矿'].forEach((name, i) =>
        addButton(landmarks, name, 'landmark', i)
      );
      section(content, '天气');
      const weather = choices();
      ['自然变化', '晴', '雨', '雷暴'].forEach((name, i) => {
        const button = addButton(weather, name, 'weather', i);
        button.setAttribute(
          'aria-pressed',
          String(s.sky.automatic ? i === 0 : ['', 'clear', 'rain', 'thunder'][i] === s.sky.weather)
        );
      });
      section(content, '时刻');
      const times = choices();
      for (const [h, label] of [
        [6, '日出'],
        [12, '正午'],
        [18, '日落'],
        [0, '午夜']
      ])
        addButton(times, label, 'hour', h);
      addButton(content, '继续飞行', 'close');
    }
    if (mode === 'journal') {
      section(content, '当前目标');
      paragraph(content, s.goal);
      section(content, '野外笔记');
      for (const [title, body] of guide) {
        const d = node('details');
        d.append(node('summary', '', title), node('p', '', body));
        content.append(d);
      }
      section(content, '世界记录');
      paragraph(
        content,
        '已采集 ' +
          s.mined +
          ' 格，建造 ' +
          s.built +
          ' 格，时间跳跃 ' +
          s.leaps +
          ' 次。种子 ' +
          s.seed +
          '。'
      );
      const links = node('div', 'voxel-links');
      content.append(links);
      addButton(links, '操作与画质', 'help');
      addButton(links, '回到研究所', 'home');
      addButton(links, '新世界', 'reset');
    }
    if (mode === 'help') {
      section(content, '画质');
      const presets = node('div', 'voxel-links');
      content.append(presets);
      ['自动', '原生精细', '流畅'].forEach((label, id) => {
        const b = addButton(presets, label, 'quality', id);
        b.setAttribute('aria-pressed', String(graphics() === ['auto', 'native', 'fast'][id]));
      });
      paragraph(
        content,
        '原生精细按屏幕实际像素绘制，保留阳光投影与斜面纹理。流畅优先保障移动和采集的帧率。'
      );
      for (const [title, body] of help) {
        section(content, title);
        paragraph(content, body);
      }
    }
    if (mode === 'talk') {
      paragraph(content, s.panel.body);
      addButton(content, '收起对话', 'close');
    }
    if (mode === 'leap') {
      paragraph(content, '把这一条线上的记忆送回 17:43。你建的方块、随身物品和收到的短信都会留下。');
      addButton(content, '戴上耳机，时间跳跃', 'leap');
    }
    if (mode === 'phone') {
      section(content, s.incoming[0]);
      paragraph(content, s.incoming[1]);
      section(content, '写给过去');
      if (!s.send.some(Boolean))
        paragraph(
          content,
          s.line === 'alpha'
            ? '发送需要启动楼下的显像管、在二楼装好中继器，并设定微波炉逆转。'
            : '这条世界线上的微波炉已经停机。'
        );
      cat.choices.forEach(([to, msg], i) => {
        if (i === 2 && !s.leaps && s.line !== 'gate') return;
        const b = addButton(content, to + '\n' + msg, 'send', i, !s.send[i]);
        b.classList.add('voxel-mail');
      });
      if (s.mail.length) {
        section(content, '已发送');
        for (const id of s.mail) paragraph(content, cat.choices[id].join(' · '));
      }
    }
    if (mode === 'reset') {
      paragraph(content, '重新生成会替换这个浏览器中的地形、建筑、背包和剧情进度。');
      addButton(content, '保留当前世界', 'journal');
      addButton(content, '确认重新生成', 'new');
    }
    if (keepScroll && old === next) content.scrollTop = scroll;
    else content.scrollTop = 0;
  }
  function state(snapshot) {
    s = snapshot;
    clock.textContent = s.number + ' %';
    root.classList.toggle('voxel-survey', s.survey);
    surveyButton.textContent = s.survey ? '返回生存' : '自由俯瞰';
    surveyButton.setAttribute('aria-pressed', String(s.survey));
    menu.querySelector('[data-action="sky"]').hidden = !s.survey;
    controls(engaged);
    menu.querySelector('[data-action="phone"]').hidden = !s.phone;
    slots.forEach((b, i) => {
      const id = s.bar[i],
        n = s.bag[id];
      b.classList.toggle('selected', i === s.slot);
      b.setAttribute('aria-pressed', String(i === s.slot));
      b.setAttribute('aria-label', i + 1 + ' ' + (n ? cat.items[id].name + ' ×' + n : '空位'));
      b.title = n ? cat.items[id].name : '空位';
      const contentKey = id + ',' + n;
      if (slotKeys[i] === contentKey) return;
      slotKeys[i] = contentKey;
      b.replaceChildren(node('small', '', String(i + 1)));
      if (n) {
        b.append(picture(id));
        if (n > 1) b.append(node('b', '', String(n)));
      }
    });
    if (s.health !== lastHealth) {
      lastHealth = s.health;
      health.textContent = '♥'.repeat(s.health) + '♡'.repeat(5 - s.health);
      health.setAttribute('aria-label', '生命 ' + s.health + ' / 5');
    }
    if (mode) panel(mode, true);
  }
  function target(hit, pct) {
    let key = '',
      label = '';
    if (hit.length) {
      const id = hit[0];
      key = id < 0 ? id + '-' + hit[1] : 'b' + id + '-' + s.bar[s.slot];
      if (id === -1) label = cat.actors[hit[1]].name + ' · ' + (coarse ? '调查' : 'E 对话');
      else if (id === -2) {
        // The offer does not change while looking at the same villager. Avoid a WASM string each frame.
        if (key !== focusKey)
          label =
            mobHint(hit[1]) +
            ' · ' +
            (hit[1] < 3 ? (coarse ? '用：交换' : 'E 交换') : coarse ? '采：击退' : '按住 F 击退');
      } else {
        const item = cat.items[id];
        label = item.name;
        const held = cat.items[s.bar[s.slot]].key;
        if (held === 'bucket' && item.key === 'water') label += ' · 取水';
        else if (held === 'water_bucket') label += ' · 倒水';
        else if (actions[item.key]) label += ' · ' + (coarse ? '用' : 'E') + ' ' + actions[item.key];
        else if (item.tier > 0) {
          const tier = ['', '木镐', '石镐', '铜镐'][item.tier];
          if (tier) label += ' · ' + tier + '以上';
        }
      }
    }
    if (key !== focusKey) {
      focusKey = key;
      focus.textContent = label;
    }
    const percent = Math.ceil(pct * 100);
    if (percent !== previousProgress) {
      previousProgress = percent;
      progress.style.width = percent + '%';
      progress.parentElement.classList.toggle('active', percent > 0);
    }
  }
  let previousSky = '';
  function position(p, elapsed) {
    const phase = (s.sky.phase + (elapsed - s.seconds) / s.sky.daySeconds) % 1;
    const mins = Math.floor(((phase + 1) % 1) * 1440);
    const text =
      String(Math.floor(mins / 60)).padStart(2, '0') +
      ':' +
      String(mins % 60).padStart(2, '0') +
      '　' +
      climates[s.sky.weather];
    if (text !== previousSky) {
      previousSky = text;
      skyReadout.textContent = text;
    }
    const [x, y, z] = [Math.floor(p[0]), Math.floor(p[1] + 0.001), Math.floor(p[2])];
    const coord = [x, y, z].join(' / ');
    const key = (s.survey ? 'fly' : 'walk') + coord;
    if (key === placeKey) return;
    placeKey = key;
    coords.textContent = coord;
    place.textContent = s.survey
      ? '自由俯瞰'
      : x >= -7 && x <= 11 && z >= -5 && z <= 6
        ? y >= 22
          ? '研究所屋顶'
          : y >= 16
            ? '未来道具研究所'
            : '布朗管工房'
        : x >= 18 && x <= 33 && z >= -9 && z <= 5
          ? '东侧露天矿'
          : Math.max(Math.abs(x), Math.abs(z)) < 32
            ? '秋叶原街区'
            : '荒野';
  }
  let engaged = false;
  function controls(active) {
    engaged = active;
    hint.textContent =
      s?.survey && !coarse
        ? 'WASD 平移　空格上升　C 下降　G 地图与天气　V 返回原地'
        : coarse
          ? ''
          : engaged
            ? 'WASD 移动　空格 跳　E 调查　B 背包'
            : '点击画面控制视角　·　WASD 移动　·　B 背包';
  }
  controls(false);
  function click(e) {
    const b = e.target.closest('button[data-action]');
    if (!b || b.disabled) return;
    if (
      touch.contains(b) &&
      (e.pointerType ||
        e.detail > 0 ||
        e.sourceCapabilities?.firesTouchEvents ||
        ['mine', 'jump', 'rise', 'descend'].includes(b.dataset.action))
    )
      return;
    handle(b.dataset.action, Number(b.dataset.id));
  }
  root.addEventListener('click', click, { signal });
  dialog.addEventListener('click', click, { signal });
  dialog.addEventListener(
    'cancel',
    (e) => {
      e.preventDefault();
      handle('close');
    },
    { signal }
  );
  return {
    root,
    stick: $('.voxel-stick'),
    touch,
    dialog,
    controls,
    storage(message) {
      $('.voxel-storage').textContent = message;
    },
    toast,
    state,
    target,
    position,
    panel,
    get mode() {
      return mode;
    },
    get coarse() {
      return coarse;
    },
    dispose() {
      clearTimeout(timeout);
      dialog.remove();
      root.remove();
    }
  };
}
