// The upstream client sends fire-and-forget websocket messages. Keep the form
// and its typed value until our HTTP endpoint acknowledges the write.
import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

export default async function adaptShared(root) {
    const world = join(root, 'sources/Game/World')
    for (const [name, action] of [
        ['Whispers', 'whispersInsert'],
        ['Areas/CircuitArea', 'circuitInsert']
    ]) {
        const path = join(world, name + '.js')
        let code = await readFile(path, 'utf8')
        const send = new RegExp(`this\\.game\\.server\\.send\\(\\{(\\s*type: '${action}',[^}]+)\\}\\)`)
        if (!code.includes('const submit = () =>') || !send.test(code)) {
            throw new Error('Missing world submission handler for ' + action)
        }
        code = code.replace('const submit = () =>', 'let submitting = false\n        const submit = async () =>')
            .replace(send, `if (submitting) return
                submitting = true
                const accepted = await this.game.server.send({$1})
                submitting = false
                if (!accepted) return`)
        if (name === 'Areas/CircuitArea') {
            // Reinitialize the daily countdown after midnight or a reconnect.
            code = code.replace("else if(data.type === 'circuitUpdate')\n            {",
                `else if(data.type === 'circuitUpdate')
            {
                if (!this.resetTime.isActive || this.resetTime.resetTime !== data.circuitResetTime) {
                    clearInterval(this.resetTime.interval)
                    this.resetTime.activate(data.circuitResetTime)
                }`)
                .replace('this.resetTime.deactivate = () =>\n        {\n            this.resetTime.isActive = true',
                    'this.resetTime.deactivate = () =>\n        {\n            this.resetTime.isActive = false')
                .replace('this.menu.updateLeaderboard(null)', 'this.menu.updateLeaderboard([])')
        }
        await writeFile(path, code)
    }

    const path = join(world, 'Areas/CookieArea.js')
    const cookies = await readFile(path, 'utf8')
    await writeFile(path, cookies.replace('this.counter.throttleUpdate()\n        }, 1000)',
        'this.counter.throttleUpdate()\n        }, 2000)'))
}
