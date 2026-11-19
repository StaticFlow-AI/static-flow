const driving = [
    ['forward', '前进', 'KeyW'], ['left', '左转', 'KeyA'],
    ['backward', '倒车', 'KeyS'], ['right', '右转', 'KeyD']
]
const radio = [
    ['toggle', '播放 / 暂停', 'KeyP'], ['previous', '上一首', 'BracketLeft'],
    ['next', '下一首', 'BracketRight'], ['rewind', '后退 5 秒', 'Comma'],
    ['advance', '快进 5 秒', 'Period']
]
const commands = [...driving, ...radio]
const defaults = Object.fromEntries(commands.map(([id, , code]) => [id, code]))
const store = 'staticflow.world.shortcuts.v1'
const reserved = new Set(['KeyB', 'KeyE', 'KeyF', 'KeyH', 'KeyL', 'KeyM', 'KeyR', 'KeyT', 'KeyV'])
const printable = { BracketLeft: '[', BracketRight: ']', Comma: ',', Period: '.', Slash: '/', Semicolon: ';', Quote: "'", Minus: '−', Equal: '=' }

function keyLabel(code) { return printable[code] || code.replace(/^(Key|Digit)/, '') }
function assignable(code) {
    return !reserved.has(code) && (/^Key[A-Z]$|^Digit[05-9]$/.test(code) || Object.hasOwn(printable, code))
}

function load() {
    try {
        const saved = JSON.parse(localStorage.getItem(store))
        const keys = commands.map(([id]) => saved[id])
        if (keys.every(assignable) && new Set(keys).size === keys.length)
            return Object.fromEntries(commands.map(([id]) => [id, saved[id]]))
    } catch { /* A fresh profile or unavailable storage uses the defaults. */ }
    return { ...defaults }
}

export class Shortcuts {
    constructor(release) {
        this.keys = load()
        this.buttons = new Map()
        const dialog = this.dialog = document.createElement('dialog')
        dialog.className = 'world-bindings'
        dialog.setAttribute('aria-labelledby', 'world-bindings-title')
        dialog.innerHTML = `<header><h2 id="world-bindings-title">驾驶台键位</h2><button type="button" data-close aria-label="关闭键位设置">×</button></header>
          <p>选中键位后按下新键。方向键始终可以驾驶。</p>
          <div class="binding-groups"></div><footer><button type="button" data-reset>恢复默认</button><span role="status"></span></footer>`
        this.notice = dialog.querySelector('[role=status]')
        for (const [label, group] of [['驾驶', driving], ['电台', radio]]) {
            const fieldset = document.createElement('fieldset')
            const legend = document.createElement('legend')
            legend.textContent = label
            fieldset.append(legend)
            for (const [id, title] of group) {
                const button = document.createElement('button')
                button.type = 'button'
                button.innerHTML = '<span></span><kbd></kbd>'
                button.firstChild.textContent = title
                button.dataset.binding = id
                button.onclick = () => {
                    this.pending = id
                    this.notice.textContent = title + ' · 按新键，Esc 取消'
                    this.refresh()
                }
                this.buttons.set(id, button)
                fieldset.append(button)
            }
            dialog.querySelector('.binding-groups').append(fieldset)
        }
        document.body.append(dialog)
        const open = () => {
            release()
            if (!dialog.open) dialog.showModal()
        }
        window.addEventListener('world-key-settings', open)
        document.querySelectorAll('[data-open-bindings]').forEach(button => button.addEventListener('click', open))
        dialog.querySelector('[data-close]').onclick = () => dialog.close()
        dialog.addEventListener('close', () => {
            this.pending = null
            this.notice.textContent = ''
            this.refresh()
        })
        dialog.querySelector('[data-reset]').onclick = () => {
            this.keys = { ...defaults }
            this.save('已恢复默认键位')
        }
        this.refresh()
    }

    refresh() {
        for (const [id, button] of this.buttons) {
            button.lastChild.textContent = this.pending === id ? '…' : keyLabel(this.keys[id])
            button.setAttribute('aria-pressed', String(this.pending === id))
        }
        const movement = driving.map(([id]) => keyLabel(this.keys[id])).join(' ')
        document.querySelectorAll('[data-driving-keys]').forEach(node => { node.textContent = movement })
    }

    save(message) {
        try { localStorage.setItem(store, JSON.stringify(this.keys)) }
        catch { message += '（仅本次有效）' }
        this.notice.textContent = message
        this.pending = null
        this.refresh()
    }

    capture(event) {
        if (!this.pending) return false
        if (event.code === 'Tab') {
            this.pending = null
            this.notice.textContent = ''
            this.refresh()
            return false
        }
        if (event.code === 'Escape') {
            this.pending = null
            this.notice.textContent = '已取消'
            this.refresh()
        } else if (!assignable(event.code) || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
            this.notice.textContent = '请选择未占用的单个字母、数字或标点键'
        } else {
            const conflict = commands.find(([id]) => id !== this.pending && this.keys[id] === event.code)
            if (conflict) this.notice.textContent = '此键用于' + conflict[1] + '，请选择其他键'
            else {
                this.keys[this.pending] = event.code
                this.save('键位已更新')
            }
        }
        return true
    }

    resolve(code) {
        const drive = driving.find(([id]) => this.keys[id] === code)
        if (drive) return { code: drive[2] }
        const media = radio.find(([id]) => this.keys[id] === code)
        if (media) return { radio: media[0] }
        // A remapped physical WASD key is no longer a driving alias.
        return { code: driving.some(([, , original]) => original === code) ? null : code }
    }
}
