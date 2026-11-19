import { v4 as uuid } from 'uuid'
import { Events } from './Events.js'
import { Game } from './Game.js'

const endpoint = '/api/world'

// The game consumes incremental scene events. The API returns a bounded,
// consistent snapshot so a sleeping tab can catch up with a single request.
export class Server {
    constructor() {
        this.game = Game.getInstance()
        this.events = new Events()
        this.connected = false
        this.initData = null
        this.queue = Promise.resolve()
        document.documentElement.classList.add('is-server-offline')
    }

    start() {
        const refresh = async () => {
            if (!document.hidden) await this.request()
            setTimeout(refresh, 5000)
        }
        refresh()
    }

    send(action) {
        if (action.type === 'whispersInsert') {
            let visitor = localStorage.getItem('staticflow-world-visitor')
            if (!visitor) {
                visitor = uuid()
                localStorage.setItem('staticflow-world-visitor', visitor)
            }
            return this.request({ ...action, visitor })
        }
        return this.request(action)
    }

    request(action) {
        const task = async () => {
            try {
                const response = await fetch(endpoint, {
                    method: action ? 'POST' : 'GET',
                    headers: action ? { 'Content-Type': 'application/json' } : {},
                    body: action ? JSON.stringify(action) : undefined,
                    cache: 'no-store',
                    signal: AbortSignal.timeout(5000)
                })
                if (!response.ok) throw new Error(String(response.status))
                const state = await response.json()
                if (state.type !== 'init') throw new Error('Invalid world snapshot')
                this.accept(state)
                return true
            } catch (error) {
                if (action) {
                    const message = error.message === '429'
                        ? '操作太频繁，请稍后再试。'
                        : '这次互动未能同步，请重试。'
                    this.game.notifications.show(message, 'world-sync', 5, null, 'world-sync')
                } else if (this.connected) {
                    this.setConnected(false)
                }
                return false
            }
        }
        this.queue = this.queue.then(task, task)
        return this.queue
    }

    setConnected(value) {
        this.connected = value
        document.documentElement.classList.toggle('is-server-online', value)
        document.documentElement.classList.toggle('is-server-offline', !value)
        this.events.trigger(value ? 'connected' : 'disconnected')
    }

    accept(state) {
        const previous = this.initData
        const reconnect = !this.connected
        this.initData = state
        if (reconnect) this.setConnected(true)
        const emit = (type, fields) => this.events.trigger('message', [{ ...fields, type }])
        if (!previous) {
            emit('init', state)
            return
        }
        if (reconnect || previous.cookiesCount !== state.cookiesCount) emit('cookiesUpdate', state)
        if (reconnect || previous.cataclysmCount !== state.cataclysmCount ||
            previous.cataclysmRunning !== state.cataclysmRunning) emit('cataclysmUpdate', state)
        if (reconnect || previous.circuitResetTime !== state.circuitResetTime ||
            JSON.stringify(previous.circuitLeaderboard) !== JSON.stringify(state.circuitLeaderboard)) {
            emit('circuitUpdate', state)
        }
        const missingFrom = (source, other) => source.filter(a => !other.some(b => a.id === b.id))
        const removed = missingFrom(previous.whispers, state.whispers)
        const added = missingFrom(state.whispers, previous.whispers)
        if (removed.length) emit('whispersDelete', { whispers: removed })
        if (added.length) emit('whispersInsert', { whispers: added })
    }
}
