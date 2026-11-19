import { Events } from '../Events.js'
import { Shortcuts } from '../../staticflow/shortcuts.js'

function consume(event) {
    event.preventDefault()
    event.stopImmediatePropagation()
}

export default class Keyboard {
    constructor(inputs) {
        this.events = new Events()
        this.held = new Map()
        this.shortcuts = new Shortcuts(() => this.release())
        window.addEventListener('keydown', event => this.down(event, inputs), true)
        window.addEventListener('keyup', event => {
            if (!this.held.has(event.code)) return
            consume(event)
            const binding = this.held.get(event.code)
            this.held.delete(event.code)
            if (binding?.key) this.events.trigger('up', [binding.key])
        }, true)
        window.addEventListener('keypress', event => {
            if (this.held.has(event.code)) consume(event)
        }, true)
        window.addEventListener('blur', () => this.release())
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) this.release()
        })
        document.addEventListener('focusin', event => {
            if (event.target.closest('input, textarea, select, [contenteditable], dialog')) this.release()
        })
    }

    release() {
        // Stop the action immediately. A repeat or keyup delivered after focus
        // returns still belongs to the press that the world originally consumed.
        for (const [code, binding] of this.held) {
            this.held.set(code, null)
            if (binding?.key) this.events.trigger('up', [binding.key])
        }
    }

    down(event, inputs) {
        // A fresh press means the physical key was released outside the page.
        if (this.held.get(event.code) === null && !event.repeat) this.held.delete(event.code)
        if (this.held.has(event.code)) { consume(event); return }
        if (event.isComposing || event.keyCode === 229) return
        if (this.shortcuts.dialog.open) {
            if (this.shortcuts.capture(event)) {
                consume(event)
                this.held.set(event.code, {})
            }
            return
        }
        if (document.querySelector('[data-worldline-dialog][open]')) return
        if (event.metaKey || event.altKey || (event.ctrlKey && event.code !== 'ControlLeft')) return
        const target = event.composedPath()[0]
        if (target?.closest?.('input:not([type=range]),textarea,select,[contenteditable]:not([contenteditable=false]),[role=textbox]') && event.code !== 'Escape') return
        if (target?.matches?.('input[type=range]') && /^(Arrow|Home|End|Page)/.test(event.code)) return
        if ((event.code === 'Enter' || event.code === 'Space') && target?.closest?.('button,a[href],summary')) return

        const { code, radio } = this.shortcuts.resolve(event.code)
        if (radio) {
            if (!inputs.game.audio?.radio || !['wandering', 'racing', 'cinematic'].some(name => inputs.filters.has(name))) return
            consume(event)
            this.held.set(event.code, {})
            const player = inputs.game.audio.radio
            if (radio === 'rewind' || radio === 'advance') player.seekBy(radio === 'rewind' ? -5 : 5)
            else player[radio]()
            return
        }
        if (!code) return
        const logical = code === event.code ? event.key : code.slice(3).toLowerCase()
        if (!event.repeat) this.events.trigger('sequence', [{ code }])
        // Dispatch one binding per physical press. Opening a dialog may
        // synchronously release it before this handler returns.
        const actions = [...inputs.actions.values()].filter(action => inputs.checkCategory(action))
        const key = [code, logical].find(alias => actions.some(action => action.keys.includes('Keyboard.' + alias)))
        if (!key) return
        consume(event)
        this.held.set(event.code, { key })
        this.events.trigger('down', [key])
    }
}
