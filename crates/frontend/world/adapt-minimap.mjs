import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

export default async function adaptMinimap(root) {
    const path = join(root, 'sources/Game/Map.js')
    let source = await readFile(path, 'utf8')
    const setup = '        this.setInputs()'
    const texture = /            const url = (this\.game\.dayCycles[^\n]+)/
    const dayNight = source.match(texture)
    if (!source.includes(setup) || !dayNight || !source.includes('    setTexture()'))
        throw new Error('Missing Folio map setup or day/night texture')
    source = "import { WorldMinimap } from '../staticflow/minimap.js'\n" + source
        .replace(setup, setup + '\n        new WorldMinimap(this)')
        .replace(texture, '            const url = this.textureUrl()')
        .replace('    setTexture()', '    textureUrl()\n    {\n        return ' + dayNight[1] + '\n    }\n\n    setTexture()')
    await writeFile(path, source)
}
