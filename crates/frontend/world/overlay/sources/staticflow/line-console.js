import './line-console.css'

const routeNames = { alpha: 'α 世界线', beta: 'β 世界线', gate: 'Steins;Gate' }

export class LineConsole {
    constructor({ onTune, onCalibrate }) {
        const dialog = this.dialog = document.createElement('dialog')
        dialog.className = 'world-line-console'
        dialog.dataset.worldlineDialog = ''
        dialog.setAttribute('aria-labelledby', 'line-console-title')
        dialog.innerHTML = `
            <header class="line-header">
                <h2 id="line-console-title"></h2>
                <form method="dialog"><button type="submit" data-line-close aria-label="关闭观测面板">×</button></form>
            </header>
            <div class="line-content">
                <div class="line-meters">
                    <div class="line-meter">
                        <span>当前变动率</span><strong data-reading></strong>
                    </div>
                    <div class="line-meter">
                        <span>公共变动率</span><strong data-public-reading></strong>
                    </div>
                </div>
                <section class="line-objective">
                    <h3>当前目标</h3>
                    <div><span data-line-route></span><p data-line-goal></p></div>
                </section>
                <div class="line-station-controls" data-station-controls>
                    <fieldset><legend>世界线调谐</legend>
                        <div class="line-channels">
                            <button type="button" data-tune="alpha"><b aria-hidden="true">α</b><span>调谐至 α 线</span><small>SERN 留下了第一封 D-Mail 的时间戳。</small></button>
                            <button type="button" data-tune="beta"><b aria-hidden="true">β</b><span>调谐至 β 线</span><small>同一晚的电文消失，广播却录下了回信。</small></button>
                        </div>
                    </fieldset>
                    <button type="button" data-calibrate>校准公共变动率</button>
                </div>
                <p class="line-journal-hint" data-journal-hint>在地图上选择「世界线观测站」，到站后操作变动率仪即可切换路线。</p>
                <section class="line-inbox">
                    <h3>D-Mail 收件箱</h3>
                    <dl class="line-receipts">
                        <div data-receipt="alpha"><dt>α 线电文</dt><dd></dd></div>
                        <div data-receipt="beta"><dt>β 线电文</dt><dd></dd></div>
                    </dl>
                    <div class="line-letters" data-line-messages></div>
                </section>
                <p class="line-ending"><span>结局</span><strong data-line-ending></strong></p>
            </div>`
        document.body.append(dialog)
        this.heading = dialog.querySelector('h2')
        this.controls = dialog.querySelector('[data-station-controls]')
        this.journalHint = dialog.querySelector('[data-journal-hint]')
        this.messages = dialog.querySelector('[data-line-messages]')

        for (const button of this.controls.querySelectorAll('[data-tune]')) {
            button.addEventListener('click', () => {
                if (!dialog.open || this.mode !== 'station') return
                onTune(button.dataset.tune)
                this.close()
            })
        }
        this.controls.querySelector('[data-calibrate]').addEventListener('click', () => {
            if (dialog.open && this.mode === 'station') onCalibrate()
        })
        // Preserve native Tab, Escape, form submission, and scrolling, while
        // keeping events inside the instrument away from world-level listeners.
        for (const name of ['click', 'keydown', 'keypress', 'keyup', 'pointerdown', 'pointerup', 'touchstart', 'touchend', 'wheel']) {
            dialog.addEventListener(name, event => event.stopPropagation())
        }

        this.map = document.querySelector('.js-modal[data-name="map"]')
        const log = document.createElement('div')
        log.className = 'line-map-log'
        log.innerHTML = `<div><span>当前目标</span><p data-map-line-goal>前往世界线观测站。</p></div>
            <button type="button" data-open-line-journal>观测记录</button>`
        this.map.append(log)
        this.mapGoal = log.querySelector('p')
        log.querySelector('button').addEventListener('click', () => {
            if (this.state) this.open('journal', this.state)
        })
    }

    open(mode, state) {
        this.mode = mode
        this.dialog.dataset.mode = mode
        const station = mode === 'station'
        this.heading.textContent = station ? '世界线观测站' : '观测记录'
        this.controls.hidden = !station
        this.journalHint.hidden = station
        this.update(state)
        if (!this.dialog.open) this.dialog.showModal()
        this.dialog.querySelector('.line-content').scrollTop = 0
    }

    close() {
        if (this.dialog.open) this.dialog.close()
    }

    update(state) {
        this.state = state
        this.map.dataset.lineRoute = state.active ?? 'idle'
        this.mapGoal.textContent = state.goal
        const dialog = this.dialog
        dialog.dataset.route = this.map.dataset.lineRoute
        dialog.querySelector('[data-line-route]').textContent = routeNames[state.active] ?? '尚未调谐'
        dialog.querySelector('[data-line-goal]').textContent = this.mode === 'station' ? state.stationGoal : state.goal
        dialog.querySelector('[data-reading]').textContent = state.reading + ' %'
        dialog.querySelector('[data-public-reading]').textContent = state.publicReading + ' %'
        for (const route of ['alpha', 'beta']) {
            const read = state[route]
            const receipt = dialog.querySelector(`[data-receipt="${route}"]`)
            receipt.dataset.read = String(read)
            receipt.querySelector('dd').textContent = read ? '已接收' : '待接收'
            dialog.querySelector(`[data-tune="${route}"]`).setAttribute('aria-pressed', String(state.active === route))
        }
        dialog.querySelector('[data-line-ending]').textContent = state.complete ? '已抵达 Steins;Gate 世界线' : '尚未抵达'
        // Public meter updates must preserve reading position and text selection.
        const contents = JSON.stringify(state.messages)
        if (contents === this.lastMessages) return
        this.lastMessages = contents
        this.messages.replaceChildren(...state.messages.map(message => {
            const letter = document.createElement('article')
            letter.dataset.kind = message.kind
            const heading = document.createElement('h4')
            heading.textContent = message.title
            const body = document.createElement('p')
            body.textContent = message.body
            letter.append(heading, body)
            return letter
        }))
    }
}
