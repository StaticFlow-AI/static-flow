import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

export default async function adaptPlaza(root) {
    const file = join(root, 'sources/Game/World/Areas/SocialArea.js')
    const source = await readFile(file, 'utf8')
    if (!source.includes('        super(model)')) throw new Error('Cannot locate social plaza model setup')
    await writeFile(file, "import { installPlazaMachine } from '../../../staticflow/plaza-machine.js'\n" +
        source.replace('        super(model)', '        installPlazaMachine(model)\n        super(model)'))
}
