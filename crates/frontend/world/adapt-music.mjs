import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

export default async function adaptMusic(root) {
    const audioPath = join(root, 'sources/Game/Audio.js')
    const audio = await readFile(audioPath, 'utf8')
    const start = audio.indexOf('        this.playlist = {}')
    const end = audio.indexOf('    setAmbiants()', start)
    const songs = audio.slice(start, end).match(/this\.playlist\.songs = (\[[\s\S]*?\n        \])/)
    if (!songs || end < 0) throw new Error('Missing Folio music playlist')
    const mount = `        this.radio = new WorldRadio(this, [...${songs[1]}, ...playlist], Boolean(import.meta.env.VITE_MUSIC))
    }

`
    let source = "import { WorldRadio } from '../staticflow/world-radio.js'\nimport { playlist } from '../staticflow/blog.js'\n" +
        audio.slice(0, start) + mount + audio.slice(end)
    const focus = /        \/\/ Tab focus \/ blur[\s\S]+?(?=\n    \}\n\n    update\(\))/
    if (!focus.test(source)) throw new Error('Missing Folio music focus handling')
    source = source.replace(focus, `        // Keep background simulation inaudible and preserve the listener's pause.
        window.addEventListener('blur', () => {
            Howler.mute(true)
            this.radio?.setSuspended(true)
        })
        window.addEventListener('focus', () => {
            Howler.mute(this.mute.active)
            this.radio?.setSuspended(false)
        })\n`)
    await writeFile(audioPath, source)

    const bowlingPath = join(root, 'sources/Game/World/Areas/BowlingArea.js')
    const bowling = await readFile(bowlingPath, 'utf8')
    if (!bowling.includes('this.game.audio.playlist.next()')) throw new Error('Missing jukebox action')
    await writeFile(bowlingPath, bowling.replace('this.game.audio.playlist.next()', 'this.game.audio.radio.next(true)'))
}
