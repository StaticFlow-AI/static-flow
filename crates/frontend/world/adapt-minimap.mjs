import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { join } from 'node:path'

export default async function adaptMinimap(root) {
    const path = join(root, 'sources/Game/Map.js')
    let source = await readFile(path, 'utf8')
    const setup = '        this.setInputs()'
    const texture = /            const url = (this\.game\.dayCycles[^\n]+)/
    const dayNight = source.match(texture)
    if (!source.includes(setup) || !dayNight || !source.includes('    setTexture()'))
        throw new Error('Missing Folio map setup or day/night texture')
    // Both maps select the same time of day. Each UI loads its own image size.
    const textureUrl = dayNight[1].replaceAll("'ui/map/", "directory + '/")
    source = "import { WorldMinimap } from '../staticflow/minimap.js'\n" + source
        .replace(setup, setup + '\n        new WorldMinimap(this)')
        .replace(texture, '            const url = this.textureUrl()')
        .replace('    setTexture()', "    textureUrl(directory = 'ui/map')\n    {\n        return " + textureUrl + '\n    }\n\n    setTexture()')
    await writeFile(path, source)

    const sharp = createRequire(join(root, 'package.json'))('sharp')
    const originals = join(root, 'static/ui/map')
    const thumbnails = join(root, 'static/ui/minimap')
    await mkdir(thumbnails, { recursive: true })
    for (const name of ['map-day', 'map-night']) {
        for (const [width, suffix] of [[384, ''], [640, '@2x']]) {
            await sharp(join(originals, name + '.png')).resize(width).webp({ quality: 84 })
                .toFile(join(thumbnails, name + suffix + '.webp'))
        }
    }
    await sharp(join(originals, 'player.png')).resize(64).webp({ quality: 86 })
        .toFile(join(thumbnails, 'player.webp'))
}
