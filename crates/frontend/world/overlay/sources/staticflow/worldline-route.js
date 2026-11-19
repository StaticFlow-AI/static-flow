// An observer keeps the two messages across shifts. Public calibration is
// shared by everyone, while a reader can finish the correspondence in any order.
import { Vector3 } from 'three/webgpu'
import { InteractivePoints } from '../Game/InteractivePoints.js'
import { articles, records } from './blog.js'
import { LineConsole } from './line-console.js'
import { createLineReceiver } from './line-receiver.js'
import lines from './world-lines.json'

const storageKey = 'staticflow.world.observer.v1'
// Ordinary calibration can cross the alpha and beta attractor fields. The
// Steins Gate reading belongs to the completed transmission.
const publicSequence = [1, 3, 2, 5, 4]
const publicLine = count => lines[publicSequence[count % publicSequence.length]]
const receivers = {
    alpha: { station: 'projects', label: 'IBN 5100 · 读取档案', floor: .02 },
    beta: { station: 'social', label: 'β 载波 · 收听广播', floor: .02 },
    gate: { station: 'landing', label: '合并两封 D-Mail', floor: .02 },
}

function readProgress() {
    try {
        const value = JSON.parse(localStorage.getItem(storageKey))
        const mail = {}
        for (const kind of ['alpha', 'beta'])
            if (typeof value.mail?.[kind] === 'string') mail[kind] = value.mail[kind]
        const complete = value.complete === true && Boolean(mail.alpha && mail.beta)
        const active = ['alpha', 'beta'].includes(value.active) || (value.active === 'gate' && complete) ? value.active : null
        return { mail, active, complete }
    } catch { return { mail: {}, active: null, complete: false } }
}

export class WorldlineRoute {
    constructor(game, onChange, onCalibrate) {
        this.game = game
        this.onChange = onChange
        this.progress = readProgress()
        this.console = new LineConsole({ onTune: kind => this.tune(kind), onCalibrate })
        this.stations = {}
        const buttons = game.inputs.interactiveButtons
        for (const [kind, { station, label, floor }] of Object.entries(receivers)) {
            const base = game.respawns.getByName(station).position.clone()
            base.set(base.x + 3, floor, base.z - 1.4)
            const model = createLineReceiver(game, base, kind)
            const point = game.interactivePoints.create(base.clone().add(new Vector3(-.9, 1.5, 1.4)), label,
                InteractivePoints.ALIGN_RIGHT, InteractivePoints.STATE_HIDDEN, () => this.receive(kind),
                () => buttons.addItems(['interact']), () => buttons.removeItems(['interact']), () => buttons.removeItems(['interact']))
            this.stations[kind] = { model, point, base, enabled: false }
        }
        window.addEventListener('storage', event => {
            if (event.key !== storageKey) return
            this.progress = readProgress()
            this.onChange()
        })
    }

    get target() {
        const { active, mail } = this.progress
        if (active === 'gate') return null
        if (mail.alpha && mail.beta) return 'gate'
        if (active && !mail[active]) return active
        return null
    }

    get signal() {
        if (this.target) return receivers[this.target].station
        return this.progress.active === 'gate' ? 'landing' : 'altar'
    }

    reading(publicCount) {
        const { active, mail } = this.progress
        if (active === 'alpha') return lines[mail.alpha ? (mail.beta ? 4 : 2) : 1]
        if (active === 'beta') return lines[mail.beta ? 5 : 3]
        if (active === 'gate') return lines[0]
        return publicLine(publicCount)
    }

    goal() {
        if (this.progress.active === 'gate') return '实验室已收到回信。你抵达了 Steins Gate 世界线。'
        if (this.target === 'gate') return '回到未来道具实验室，在双频发信器合并两封 D-Mail。'
        if (this.target === 'alpha') return '去文章工坊，用 IBN 5100 读出归档电文的时间戳。'
        if (this.target === 'beta') return '去广播会馆，用短波接收机找到回信的频率。'
        if (this.progress.mail.alpha) return '回观测站调谐 β 世界线，寻找另一半回信。'
        if (this.progress.mail.beta) return '回观测站调谐 α 世界线，寻找另一半回信。'
        return '去世界线观测站，选择接收 α 或 β 的第一封 D-Mail。'
    }

    state(publicCount) {
        const { active, mail, complete } = this.progress
        const messages = []
        if (mail.alpha) messages.push({ kind: 'alpha', title: 'α 来信 · 档案的时间戳', body: mail.alpha })
        if (mail.beta) messages.push({ kind: 'beta', title: 'β 来信 · 沉默的载波', body: mail.beta })
        if (complete) messages.push({ kind: 'gate', title: '来自另一侧的回信', body: '你在 α 留下原来的归档时间，把正文送进 β 广播的空拍。观察记录一字未改，收件人却读到了另一种结局。桌上的微波炉响了一声：1.048596%。El Psy Kongroo。' })
        const stationGoal = !active ? '任选一个频道调谐，接收第一封 D-Mail。' :
            this.signal === 'altar' ? '调到另一条世界线，寻找回信缺少的线索。' : this.goal()
        return { active, alpha: Boolean(mail.alpha), beta: Boolean(mail.beta), complete, messages,
            reading: this.reading(publicCount)[0], publicReading: publicLine(publicCount)[0], goal: this.goal(), stationGoal }
    }

    refresh(publicCount) {
        this.publicCount = publicCount
        for (const [kind, station] of Object.entries(this.stations)) {
            const enabled = kind === this.target
            if (enabled === station.enabled) continue
            station.enabled = enabled
            station.model.setVisible(enabled)
            const { point } = station
            if (this.game.interactivePoints.temporaryHidden)
                point.recoveryState = enabled ? InteractivePoints.STATE_CONCEALED : InteractivePoints.STATE_HIDDEN
            else if (enabled) point.show()
            else point.hide()
        }
        this.console.update(this.state(publicCount))
    }

    save() {
        try { localStorage.setItem(storageKey, JSON.stringify(this.progress)) } catch { /* The current visit still works. */ }
        this.onChange()
    }

    tune(kind) {
        if (kind !== 'alpha' && kind !== 'beta') return
        this.progress.active = kind
        this.save()
        this.game.audio.groups.get('click').play(true)
        this.game.notifications.show(this.goal(), 'line-tune', 6)
    }

    openStation() { this.console.open('station', this.state(this.publicCount)) }

    receive(kind) {
        if (kind !== this.target) return
        if (kind === 'alpha') {
            const article = articles[0]
            const title = article ? '《' + article.fullTitle + '》' : '这份档案'
            this.progress.mail.alpha = 'IBN 5100 从' + title + '的归档头读出一条 SERN 截获记录。无论重发几次，第一封 D-Mail 的时间戳总落在同一秒。“把那一秒留下。试着改动它里面发生的事。”'
        } else if (kind === 'beta') {
            const song = records[0]?.image.name
            const title = song ? '《' + song + '》' : '节目'
            this.progress.mail.beta = '广播会馆的旧接收机在' + title + '的尾声收到一段窄带信号。录音里只有一句话：“把频率交给记得时间的人。”你把载波抄在了磁带盒背面。'
        } else {
            this.progress.complete = true
            this.progress.active = 'gate'
        }
        this.save()
        this.game.audio.groups.get('click').play(true)
        this.console.open('journal', this.state(this.publicCount))
    }
}
