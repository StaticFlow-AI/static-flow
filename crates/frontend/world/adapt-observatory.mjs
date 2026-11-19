import { copyFile, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

export default async function adaptObservatory(root) {
    const path = join(root, 'sources/Game/World/Areas/Areas.js')
    const source = await readFile(path, 'utf8')
    const original = "import { AltarArea } from './AltarArea.js'"
    if (!source.includes(original)) throw new Error('Missing altar area registration')
    await writeFile(path, source.replace(original, "import { WorldlineObservatory as AltarArea } from '../../../staticflow/observatory.js'"))
    const game = join(root, 'sources/Game/Game.js')
    await writeFile(game, (await readFile(game, 'utf8')).replace(/^.*\[ 'satanStarTexture'.*\n/m, ''))
    await copyFile(new URL('../world-lines.json', import.meta.url), join(root, 'sources/staticflow/world-lines.json'))
}
