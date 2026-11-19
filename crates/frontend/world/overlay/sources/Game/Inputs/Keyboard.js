import { Events } from '../Events.js'
import { Shortcuts } from '../../staticflow/shortcuts.js'

function consume(event) {
    event.preventDefault()
    event.stopImmediatePropagation()
}

export default class Keyboard {
    constructor(inputs) {
        this.inputs = inputs
        this.events = new Events()
        this.held = new Map()
        this.shortcuts = new Shortcuts(() => this.release())
        // Key events do not bubble across browsing contexts. A focused game
        // control thus owns the press even when media listeners ran first on window.
        const frame = this.frame = document.createElement('iframe')
        frame.className = 'world-driving-frame'
        frame.title = '驾驶控制'
        frame.hidden = true
        inputs.game.canvasElement.after(frame)
        const surface = this.surface = frame.contentDocument.createElement('input')
        surface.readOnly = true
        surface.inputMode = 'none'
        surface.setAttribute('aria-label', '驾驶控制，按 Tab 切换到页面控件')
        frame.contentDocument.body.append(surface)

        let pointerInteraction = false
        let returnToDriving = false
        document.addEventListener('pointerdown', () => { pointerInteraction = true }, true)
        for (const context of [window, frame.contentWindow]) {
            context.addEventListener('keydown', event => {
                pointerInteraction = false
                this.down(event, inputs)
            }, true)
            context.addEventListener('keyup', event => {
                if (!this.held.has(event.code)) return
                consume(event)
                const binding = this.held.get(event.code)
                this.held.delete(event.code)
                if (binding?.key) this.events.trigger('up', [binding.key])
            }, true)
            context.addEventListener('keypress', event => {
                if (this.held.has(event.code)) consume(event)
            }, true)
            context.addEventListener('blur', () => this.release())
        }
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) this.release()
        })
        document.addEventListener('focusin', event => {
            if (event.target.closest('dialog') && !event.relatedTarget?.closest?.('dialog'))
                returnToDriving = pointerInteraction || event.relatedTarget === frame
            if (event.target.closest('input, textarea, select, [contenteditable], dialog')) this.release()
        })
        window.addEventListener('world-driving-focus', () => this.focusDriving())
        // Native dialogs restore focus to their launcher. Give a pointer user
        // back the wheel, while keeping a keyboard user's tab stop on the launcher.
        // The native close event is queued for a later frame. Observe open
        // being removed so focus is restored before the very next keypress.
        new MutationObserver(changes => {
            if (!changes.some(({ target }) => target.tagName === 'DIALOG' && !target.open)) return
            if (returnToDriving || document.activeElement === frame || document.activeElement === document.body)
                this.focusDriving()
            returnToDriving = false
        }).observe(document.body, { subtree: true, attributes: true, attributeFilter: ['open'] })
        document.addEventListener('click', event => {
            if (event.detail && (event.target === inputs.game.canvasElement || document.activeElement === document.body))
                this.focusDriving()
        })
    }

    onFilterChange() {
        this.frame.hidden = !['wandering', 'racing', 'cinematic'].some(name => this.inputs.filters.has(name))
        if (!this.frame.hidden) this.focusDriving(true)
    }

    focusDriving(leavingPanel = false) {
        if (this.frame.hidden || document.querySelector('dialog[open]')) return
        const active = document.activeElement
        if (active?.matches('input,textarea,select,[contenteditable]:not([contenteditable=false]),[role=textbox]')
            && !(leavingPanel && active.closest('.js-menu,.js-modals'))) return
        this.surface.focus({ preventScroll: true })
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
        if (event.target === this.surface && event.key === 'Enter' && event.altKey &&
            !event.ctrlKey && !event.metaKey && !event.shiftKey) {
            consume(event)
            if (!event.repeat) {
                this.held.set(event.code, {})
                window.dispatchEvent(new Event('world-fullscreen-shortcut'))
            }
            return
        }
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
        if (target !== this.surface && target?.closest?.('input:not([type=range]),textarea,select,[contenteditable]:not([contenteditable=false]),[role=textbox]') && event.code !== 'Escape') return
        if (event.code === 'Escape' && (document.fullscreenElement || document.webkitFullscreenElement)) return
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
