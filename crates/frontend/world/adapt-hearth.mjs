import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

export default async function adaptHearth(root) {
    const areaPath = name => join(root, 'sources/Game/World/Areas', name + 'Area.js')
    const landing = await readFile(areaPath('Landing'), 'utf8')
    const start = landing.indexOf('        // Hashes\n')
    const end = landing.indexOf('        // Burn\n', start)
    if (start < 0 || end < 0) throw new Error('Missing landing bonfire effects')
    await writeFile(areaPath('Landing'),
        "import { installPhoneWave } from '../../../staticflow/lab-hearth.js'\n" +
        (landing.slice(0, start) + landing.slice(end))
            .replace('        super(model)', '        installPhoneWave(model)\n        super(model)')
            .replace("get('bonfireHashes')", "get('phoneWaveVent')")
            .replace("get('bonfireBurn')", "get('phoneWaveHeater')")
            .replace('Math.pow(Math.random(), 1.5) * 1', 'Math.pow(Math.random(), 1.5) * 0.12')
            .replace("this.game.audio.groups.get('campfire').items[0].positions.push(position)",
                "const sources = this.game.audio.groups.get('campfire').items[0].positions\n                    if (!sources.includes(position)) sources.push(position)"))
    const cookie = await readFile(areaPath('Cookie'), 'utf8')
    await writeFile(areaPath('Cookie'),
        "import { installLaboratoryOven } from '../../../staticflow/lab-hearth.js'\n" +
        cookie.replace('        super(model)', '        installLaboratoryOven(model)\n        super(model)'))
}
