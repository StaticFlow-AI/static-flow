import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'

export default async function adaptContent(root) {
    const source = path.join(root, 'sources')
    // Match the pinned version explicitly so an upstream refactor cannot silently
    // ship original portfolio links or disconnect a building from the catalog.
    async function edit(file, changes, imports = '') {
        let contents = await readFile(path.join(source, file), 'utf8')
        for (const [from, to] of changes) {
            if (typeof from === 'string' ? !contents.includes(from) : !from.test(contents))
                throw new Error('World content adapter: missing match in ' + file + ' for ' + from)
            contents = contents.replaceAll(from, to)
        }
        await writeFile(path.join(source, file), imports + contents)
    }
    await edit('Game/Game.js', [
        ['// First batch for intro', 'const blogReady = loadBlog()\n\n        // First batch for intro'],
        ['Promise.all([ resourcesPromise, rapierPromise ])', 'Promise.all([ resourcesPromise, rapierPromise, blogReady ])'],
        [/^\s*\[ '(?:career(?:Freelancer|Hetic|ImmersiveGarden|IRLTeacher|OnlineTeacher|Uzik)|timeMachineScreen(?:MGS|Folio))Texture'.*$/gm, ''],
    ], "import { loadBlog } from '../staticflow/blog.js'\n")

    for (const [file, name] of [['projects', 'articles'], ['lab', 'records'], ['social', 'destinations']])
        await writeFile(path.join(source, 'data', file + '.js'), "export { " + name + " as default } from '../staticflow/blog.js'\n")

    await edit('data/achievements.js', [[/export default\s*\[/g, 'export default () => [']])
    await edit('Game/Achievements.js', [['of achievementsData)', 'of achievementsData())']])
    const areas = 'Game/World/Areas/'
    for (const [area, newName, action] of [['Projects', '文章工坊', '阅读文章'], ['Lab', '唱片实验室', '打开播放器']]) {
        const changes = [
            ["'" + area + "',\n            InteractivePoints.", "'" + newName + "',\n            InteractivePoints."],
            [/            const path = `(?:projects|lab)\/images\/\$\{key\}`\s*/g, '            '],
            [/const loader = this\.game\.resourcesLoader\.getLoader\('textureKtx'\)\s*loader\.load\(\s*path,\s*\(loadedTexture\) =>\s*\{[\s\S]*?this\.images\.loadEnded\(key\)\s*\}\s*\)/g,
                'loadBoard(key, (texture) => {\n                    resource.texture = texture\n                    resource.loaded = true\n                    this.images.loadEnded(key)\n                })'],
            [/this\.navigation\.current\.url\.replace\([^\n]+?\)/g, JSON.stringify(action)],
            ["window.open(this.navigation.current.url, '_blank')", "window.open(this.navigation.current.url, '_blank', 'noopener')"],
            ["setProgress('" + area.toLowerCase() + "', this.navigation.current.title)", "setProgress('" + area.toLowerCase() + "', this.navigation.current.url)"],
        ]
        if (area === 'Lab') changes.push([
            /const loader = this\.game\.resourcesLoader\.getLoader\('textureKtx'\)\s*loader\.load\(\s*`lab\/images\/\$\{project\.imageMini\}`,/g,
            'loadBoard(project.imageMini,',
        ])
        await edit(areas + area + 'Area.js', changes, "import { loadBoard } from '../../../staticflow/blogBoards.js'\n")
    }

    await edit(areas + 'SocialArea.js', [
        ['this.setLinks()', 'dressDirectory(this, socialData)\n        this.setLinks()'],
        [/if\(link\.url\)\s*window.open\(link.url, '_blank'\)\s*else\(link.modal\)\s*this.game.modals.open\(link.modal\)/g, "window.open(link.url, '_blank', 'noopener')"],
        ["'OnlyFans',", "'风扇喷泉',"],
    ], "import { dressDirectory } from '../../../staticflow/blogBoards.js'\n")
    await edit(areas + 'BehindTheSceneArea.js', [["'Behind the scene',", "'阅读室',"]])
    await edit(areas + 'CareerArea.js', [
        ['line.texture = this.game.resources[`${line.group.userData.texture}Texture`]', `line.category = categories[this.lines.items.length % categories.length]
            line.texture = inscription(line.category.name)
            const point = group.position.clone()
            point.y = 1.2
            this.game.interactivePoints.create(point, line.category.name, InteractivePoints.ALIGN_RIGHT,
                InteractivePoints.STATE_CONCEALED, () => window.open(line.category.url, '_blank', 'noopener'),
                () => this.game.inputs.interactiveButtons.addItems(['interact']),
                () => this.game.inputs.interactiveButtons.removeItems(['interact']),
                () => this.game.inputs.interactiveButtons.removeItems(['interact']))`],
        ['this.year.start = 2008', 'this.year.start = 0'],
        ['const yearString = `${year}`', "const yearString = String(year).padStart(4, '0').slice(-4)"],
        ['const yearCurrent = this.year.start + Math.floor(this.year.offsetTarget)', `const nearest = this.lines.items.reduce((a, b) =>
            a.origin.distanceToSquared(this.game.player.position) <= b.origin.distanceToSquared(this.game.player.position) ? a : b)
        const yearCurrent = nearest.category.count`],
    ], "import { categories } from '../../../staticflow/blog.js'\nimport { inscription } from '../../../staticflow/blogBoards.js'\nimport { InteractivePoints } from '../../InteractivePoints.js'\n")

    await edit(areas + 'TimeMachineArea.js', [
        ['this.setInteractivePoint()', 'this.years = [2025, 2024]\n        this.archiveIndex = 0\n        this.setInteractivePoint()'],
        ["'Time Machine',", "'年度回顾',"],
        ["window.open('https://2019.bruno-simon.com')", "window.open(sitePath('/standalone/github-wrapped-' + this.years[this.archiveIndex] + '.html'), '_blank', 'noopener')"],
        [/const screenTextures = \[\s*this\.game\.resources\.timeMachineScreenFolioTexture,\s*this\.game\.resources\.timeMachineScreenMGSTexture,\s*\]/g,
            "const screenTextures = this.years.map(year => inscription('GitHub · ' + year))"],
        ['collideIndex++', 'collideIndex++\n                this.archiveIndex = collideIndex % this.years.length'],
    ], "import { sitePath } from '../../../staticflow/blog.js'\nimport { inscription } from '../../../staticflow/blogBoards.js'\n")
    await edit('Game/Audio.js', [
        ['this.playlist.index = (Math.floor', 'this.playlist.songs.push(...playlist)\n        this.playlist.index = (Math.floor'],
        ['src: [ song.path ],', 'src: [ song.path ],\n                format: song.format ? [song.format] : undefined,\n                html5: Boolean(song.format),'],
        ['const html = /* html */`\n                    <div class="top">\n                        <div class="title">Now playing', 'const escaped = document.createElement(\'span\')\n                escaped.textContent = this.playlist.current.name\n                const html = /* html */`\n                    <div class="top">\n                        <div class="title">正在播放'],
        ['${this.playlist.current.name}', '${escaped.innerHTML}'],
    ], "import { playlist } from '../staticflow/blog.js'\n")
}
