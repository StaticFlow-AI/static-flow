// The world bundle is assembled from a pinned upstream checkout. Keep its
// event-bound DOM nodes and game assets while replacing the visitor-facing UI.
import { copyFile, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { localizeAchievements } from './achievements-zh.mjs'

const destinations = {
    'Achievements': '成就殿堂', 'Altar': '祭坛', 'Behind<br /> the scene': '阅读室',
    'Bowling': '保龄球场', 'Career': '分类长廊', 'Circuit': '赛道', 'Cookie': '饼干屋',
    'Lab': '唱片实验室', 'Landing': '驾驶营地', 'Projects': '文章工坊',
    'Social': '站内导航', 'Time Machine': '年度回顾',
}

const tabLabels = {
    home: '驾驶营地', options: '设置', controls: '驾驶指南', achievements: '探索成就',
    circuit: '赛道', whispers: '路边留言', behindTheScene: '阅读室',
}

const labels = {
    Interact: '互动', Unstuck: '脱困', Audio: '声音', Quality: '画质', Options: '驾驶设置',
    'Toggles sound': '打开或关闭声音', Low: '低', 'Toggles some effects': '切换画面特效',
    "I'm stuck!": '车辆脱困', Respawn: '回到重生点', 'Teleports you to the closest respawn': '回到最近的重生点',
    Reset: '复位', 'Resets every object': '将所有物件复位', Renderer: '渲染器',
    'Best for performance': '当前使用 WebGPU 渲染', Server: '世界服务', Pending: '正在连接',
    'Mouse Keyboard': '键盘鼠标', 'Mobile Tablet': '触摸屏', Gamepad: '手柄',
    'Move around': '驾驶', Boost: '加速', Brake: '刹车', Jump: '跳跃', Map: '地图',
    Mute: '静音', 'Post a whisper': '留下路边留言', 'Activate hydraulics': '控制悬挂',
    'Move camera': '移动视角', Honk: '鸣笛', 'One finger': '单指拖动', 'Two fingers': '双指',
    'Move the car': '驾驶', 'Move camera / zoom': '移动视角或缩放', 'Tap (on the car)': '轻点小车',
    'Interact / Exit': '互动或退出', Accelerate: '前进', 'Backward accelerate': '倒车',
    Hydraulics: '悬挂', 'Joystick Left': '左摇杆', 'Joystick Left (press)': '按下左摇杆',
    'Joystick Right': '右摇杆', 'Joystick Right (press)': '按下右摇杆', 'Turn wheels': '转向',
    'Zoom in/out': '缩放', Pause: '暂停', Achievements: '探索成就', Rewards: '解锁涂装',
    'Unlock at': '所需成就', 'Reset achievements': '清除探索记录', Circuit: '赛道排行榜',
    "Server currently offline. Scores can't be saved.": '当前为单人模式。圈速仅在本次驾驶中显示。',
    'No score yet today': '今天还没有圈速记录', 'Resets in': '距离下一次重置',
    Restart: '重新起跑', End: '结束比赛', Controls: '驾驶指南', 'Leave a whisper': '留下一句话',
    'Whispers are messages left by visitors.': '把留言留在小车停靠的地方，经过的驾驶员可以读到它。',
    'Server currently offline': '留言服务当前未连接。', 'Your message here': '在这里留言',
    'No result': '没有找到', 'Your time': '本圈用时', Submit: '提交', or: '或',
    "Sorry, you didn't make it to the top 10.": '这次未进前十，再跑一圈试试？',
    'Private messages': '读者共建台', 'Public server': '文章与图片', 'Join server': '打开图库', 'Start chating': '前往共建台',
}

function replace(source, anchor, replacement) {
    const found = typeof anchor === 'string' ? source.includes(anchor) : anchor.test(source)
    if (!found) throw new Error('Missing UI anchor: ' + anchor)
    return source.replace(anchor, replacement)
}

const opening = `
    <a class="world-return" href="../../" aria-label="返回 StaticFlow 博客首页">
        <span class="arrow" aria-hidden="true">←</span> 返回博客
    </a>
    <section class="world-brief" aria-labelledby="world-title">
        <h1 id="world-title">StaticFlow 漫游志</h1>
        <p id="world-status" role="status">正在装载驾驶营地…</p>
        <button id="world-drive" type="button" hidden>开始驾驶</button>
        <button id="world-retry" type="button" hidden onclick="location.reload()">重新载入</button>
        <noscript><p>请启用 JavaScript 进入 3D 世界。</p></noscript>
    </section>`

const home = `<div class="title">驾驶营地</div>
    <p class="text">这里是 ackingliu 的博客。开过营地的篝火，去文章工坊翻阅最新的工程手记，也可以在唱片实验室找首歌听。</p>
    <p class="text">用 WASD 或方向键驾驶，靠近标记后按回车。触屏上可单指拖动，轻点浮出的按钮互动。打开地图，选择地名即可传送。</p>
    <nav class="world-links text" aria-label="博客馆藏">
        <a href="../../posts">浏览知识库 ↗</a><a href="../../media/audio">进入曲库 ↗</a>
    </nav>
    <div data-sf-articles></div>`

const reading = `<div class="title">阅读室</div>
    <p class="text">在这里翻阅博客最新收录的文章。</p>
    <div data-sf-articles></div>
    <h2 class="title">馆藏图片</h2>
    <div data-sf-gallery></div>
    <p class="text"><a href="../../media/image">浏览图库</a> · <a href="../../posts">全部文章</a> · <a href="../../categories">按分类查阅</a></p>
    <h2 class="title">关于这个世界</h2>
    <p class="text">本站用 Rust、Axum、Yew 和 LanceDB 管理文章、音乐与图片。漫游世界通过同源接口读取这些馆藏。</p>
    <p class="text">驾驶、碰撞和场景基于 Bruno Simon 以 MIT 许可开放的 <a href="https://github.com/brunosimon/folio-2025" target="_blank" rel="noopener noreferrer">Folio 2025</a>。Three.js 绘制世界，Rapier 负责物理模拟，Howler.js 播放声音。原场景配乐来自 Kounine。</p>`

export default async function adaptUI(root) {
    const edit = async (path, update) => {
        const filename = join(root, 'sources', path)
        await writeFile(filename, update(await readFile(filename, 'utf8')))
    }

    await edit('index.html', source => {
        let html = replace(source, '<html lang="en">', '<html lang="zh-CN">')
        html = replace(html, /<title>[\s\S]*?(?=<link rel="stylesheet" href="\.\/style\/index\.styl">)/,
            '<title>StaticFlow 漫游志</title>\n    <meta name="description" content="开车逛 ackingliu 的博客，沿途阅读文章、试听音乐、参加计时赛。">\n    ')
        html = replace(html, /<link rel="icon"[\s\S]*?(?=<link rel="preload")/,
            '<link rel="stylesheet" href="./staticflow/world.css">\n    <link rel="icon" href="./favicon.ico">\n    ')
        html = replace(html, /<!-- Analytics -->[\s\S]*?<\/script>\s*<script>[\s\S]*?<\/script>/, '')
        html = html.replaceAll('rel="preload" href="/', 'rel="preload" href="./')
        html = replace(html, '<body>', '<body>' + opening)
        html = replace(html, '<canvas class="js-canvas"></canvas>', '<canvas aria-label="可驾驶的博客 3D 世界" class="js-canvas"></canvas>')
        html = replace(html, /<div class="title">Bruno's Home[\s\S]*?And don't break anything!<\/p>/, home)
        html = replace(html, /<div class="title">Behind the scene[\s\S]*?— Bruno\s*<\/p>/, reading)
        html = replace(html, /<div class="title"><img src="ui\/discord.svg" loading="lazy"> Discord<\/div>/, '<div class="title">站内导航</div>')
        html = replace(html, /<p>Come hang out[\s\S]*?<\/p>/, '<p>漫游之外，看看博客收录的图片与插画。</p>')
        html = replace(html, /<p>Contact me[\s\S]*?<\/p>/, '<p>把想读的文章或想听的歌放进读者共建台。</p>')
        html = replace(html, /https:\/\/discord.com\/channels\/[\d/]+/, '../../media/image')
        html = replace(html, /https:\/\/discord.com\/users\/\d+/, '../../studio')
        html = replace(html, /- Everyone can see them[\s\S]*?- Max 30 characters/,
            '公开留言最多 30 个字符。可以附上一面旗帜。每位驾驶员保留一条，世界最多保留最近 30 条。')
        html = html.replaceAll('placeholder="Your message"', 'placeholder="写下你的留言"')
            .replaceAll('placeholder="Search…"', 'placeholder="查找旗帜…"')
            .replaceAll('src="ui/previews/options.webp"', 'src="ui/previews/home.webp"')
            .replaceAll('src="ui/flags/fr.webp"', 'alt="" src="ui/flags/fr.webp"')
            .replaceAll('<button class="js-close close"', '<button aria-label="关闭" class="js-close close"')
            .replaceAll('<button class="js-map-trigger map-trigger"', '<button aria-label="打开地图" title="地图 (M)" class="js-map-trigger map-trigger"')
            .replaceAll('<button class="js-menu-trigger menu-trigger"', '<button aria-label="打开驾驶手册" title="驾驶手册" class="js-menu-trigger menu-trigger"')
            .replaceAll('<button class="submit has-icon">', '<button aria-label="发布留言" class="submit has-icon">')
        html = html.replace(/<button class="js-navigation-item item" data-name="(\w+)">/g,
            (_, key) => `<button class="js-navigation-item item" data-name="${key}" title="${tabLabels[key]}" aria-label="${tabLabels[key]}">`)
        const touch = { previous: '上一项', next: '下一项', open: '打开', close: '关闭' }
        html = html.replace(/<button data-name="(previous|next|open|close)"/g,
            (_, key) => `<button aria-label="${touch[key]}" data-name="${key}"`)
        // Upstream lists the trigger buttons backwards. Player.forward uses r2.
        html = html.replace(/>(LT|L2|RT|R2)<\/span>/g, (_, key) =>
            '>' + ({ LT: 'RT', L2: 'R2', RT: 'LT', R2: 'L2' })[key] + '</span>')
        return html.replace(/>([^<>]+)</g, (match, text) => {
            const translation = labels[text.trim()]
            return translation ? '>' + text.replace(text.trim(), translation) + '<' : match
        })
    })
    await copyFile(new URL('../static/favicon.ico', import.meta.url), join(root, 'static/favicon.ico'))

    await edit('Game/Map.js', source => {
        for (const [name, display] of Object.entries(destinations))
            source = replace(source, `name: '${name}'`, `name: '${display}'`)
        return replace(source, "document.createElement('div')", "document.createElement('button')")
            .replace(/\s*element.addEventListener\('keydown',[\s\S]*?\n        }\)/, '')
    })
    await edit('Game/Menu.js', source => replace(source, /\s*element.addEventListener\('keydown',[\s\S]*?\n        }\)/, ''))
    await edit('Game/Title.js', source => replace(source, "document.title = 'Bruno'", "document.title = 'StaticFlow 漫游志'"))
    await edit('Game/Options.js', source => source.replaceAll("'High' : 'Low'", "'高' : '低'")
        .replace("'Online'", "'在线'").replace("'Offline'", "'单人模式'")
        .replace('Enjoy the <strong>multiplayer</strong> features', '已连接到共享世界')
        .replace('Should be back soon', '当前可以单人驾驶')
        .replace('Your browser is <strong>not compatible</strong> with WebGPU resulting in performance loss', '浏览器正在使用 WebGL 渲染'))
    await edit('Game/Materials.js', source => {
        const translated = replace(source, 'alphaNode = texture(baseMaterial.map).r', 'baseColor = localizedLabelColor(baseMaterial)\n                alphaNode = baseColor.r')
        return "import { localizedLabelColor } from '../staticflow/labels.js'\n" + translated
    })
    await edit('Game/World/Intro.js', source => {
        source = replace(source, /            \/\/ Load, set and save texture[\s\S]*?(?=\n        }\n\n        this.text.updateTexture\(\))/, `            if (!this.text.textures.has(name))
                this.text.textures.set(name, drivePrompt(name))
            const prompt = this.text.textures.get(name)
            material.outputNode = Fn(() => {
                texture(prompt, vec2(uv().x, uv().y.oneMinus())).r.lessThan(0.5).discard()
                return vec4(1)
            })()
            material.needsUpdate = true
            mesh.visible = true`)
        // The original async loader completed after material and mesh creation.
        source = replace(source, '        this.text.updateTexture()\n', '')
        source = replace(source, '        this.text.mesh = mesh', '        this.text.mesh = mesh\n        this.text.updateTexture()')
        return "import { drivePrompt } from '../../staticflow/labels.js'\n" + source
    })
    await edit('Game/World/Areas/LandingArea.js', source => {
        source = "import { installCampSign } from '../../../staticflow/camp-sign.js'\n" + source
        return replace(source, 'super(model)', 'installCampSign(model)\n        super(model)')
            .replace("'Map'", "'地图'").replace("'Controls'", "'驾驶指南'").replace("'Res(e)t'", "'篝火复位'")
    })
    await edit('Game/Game.js', source => replace(source, 'this.init()', `this.init().catch(error => {
            console.error('StaticFlow world initialization failed', error)
            document.documentElement.classList.add('world-failed')
            document.getElementById('world-status').textContent = '世界未能载入。请检查网络和浏览器的图形加速设置，再试一次。'
            document.getElementById('world-drive').hidden = true
            document.getElementById('world-retry').hidden = false
        })`))
    await edit('Game/Reveal.js', source => {
        source = replace(source, 'const next = () =>', "const startButton = document.getElementById('world-drive')\n                    const next = () =>")
        source = replace(source, 'this.game.rayCursor.removeIntersect(intersect)', "this.game.rayCursor.removeIntersect(intersect)\n                        startButton.removeEventListener('click', next)\n                        startButton.hidden = true")
        return replace(source, "this.game.inputs.events.on('introStart', inputCallback)", `this.game.inputs.events.on('introStart', inputCallback)
                    document.getElementById('world-status').textContent = matchMedia('(pointer: coarse)').matches
                        ? '单指拖动驾驶，轻点小车跳跃。' : 'WASD 驾驶 · 空格跳跃 · 回车互动 · M 地图'
                    startButton.hidden = false
                    startButton.addEventListener('click', next)`)
    })
    await edit('data/achievements.js', localizeAchievements)
    await edit('Game/Achievements.js', source => source.replace("'Are you sure?'", "'确定清除？'")
        .replace("'Definitely?'", "'再次点击确认'").replace("'Done!'", "'已清除'").replace("'Reset achievements'", "'清除探索记录'"))
    const prompts = {
        Achievements: { Achievements: '探索成就' }, Circuit: { 'Start race!': '开始计时赛' },
        Cookie: { 'Accept cookie': '领一块饼干' },
        Bowling: { Restart: '重新摆瓶', Bumpers: '切换护栏', 'Change song': '换首歌' },
    }
    for (const [area, names] of Object.entries(prompts)) {
        await edit('Game/World/Areas/' + area + 'Area.js', source => {
            if (area === 'Circuit') source = source.replace("'OFFLINE'", "'单人模式'").replace("'NO SCORE YET TODAY'", "'今日尚无成绩'")
            for (const [label, name] of Object.entries(names))
                source = replace(source, "'" + label + "',\n            InteractivePoints.", "'" + name + "',\n            InteractivePoints.")
            return source
        })
    }
    await edit('Game/World/Whispers.js', source => source.replaceAll('Your message here', '在这里留言'))
}
