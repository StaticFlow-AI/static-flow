import { Howl } from 'howler'
import './world-radio.css'

const transport = `
    <div class="radio-head">
        <div class="radio-title" aria-live="polite"></div>
        <div class="radio-buttons">
            <button type="button" data-action="previous" aria-label="上一首" title="上一首"><i class="radio-back" aria-hidden="true"></i></button>
            <button type="button" data-action="toggle" aria-label="播放音乐"><i class="radio-play" aria-hidden="true"></i></button>
            <button type="button" data-action="next" aria-label="下一首" title="下一首"><i class="radio-next" aria-hidden="true"></i></button>
        </div>
    </div>
    <span class="radio-status" aria-live="polite"></span>
    <div class="radio-tuning">
        <span class="radio-elapsed"></span>
        <input type="range" class="radio-range" min="0" max="0" value="0" step="1" aria-label="音乐进度">
        <span class="radio-duration"></span>
        <button type="button" id="world-key-settings" aria-label="设置快捷键" title="设置快捷键"><span class="radio-keyboard" aria-hidden="true"></span></button>
    </div>`

function clock(seconds) {
    const whole = Math.floor(seconds)
    return Math.floor(whole / 60) + ':' + String(whole % 60).padStart(2, '0')
}

export class WorldRadio {
    constructor(audio, songs, autoplay) {
        this.audio = audio
        this.songs = songs
        this.suspended = false
        this.scrubbing = false

        this.element = document.createElement('section')
        this.element.className = 'world-radio'
        this.element.setAttribute('aria-label', '车载电台')
        this.element.innerHTML = transport
        document.body.append(this.element)
        this.title = this.element.querySelector('.radio-title')
        this.status = this.element.querySelector('.radio-status')
        this.toggleButton = this.element.querySelector('[data-action="toggle"]')
        this.range = this.element.querySelector('.radio-range')
        this.elapsed = this.element.querySelector('.radio-elapsed')
        this.duration = this.element.querySelector('.radio-duration')

        for (const button of this.element.querySelectorAll('button')) {
            button.addEventListener('click', event => {
                if (button.id === 'world-key-settings') {
                    window.dispatchEvent(new Event('world-key-settings'))
                } else {
                    this[button.dataset.action]()
                    // After a pointer click, the next driving key goes to the car.
                    // Keyboard users keep their tab stop.
                    if (event.detail) button.blur()
                }
            })
        }

        this.range.addEventListener('input', () => {
            this.scrubbing = true
            this.paintTime(this.range.valueAsNumber, Number(this.range.max))
        })
        this.range.addEventListener('change', () => {
            this.scrubbing = false
            this.seekTo(this.range.valueAsNumber)
        })
        this.range.addEventListener('blur', () => { this.scrubbing = false })
        this.range.addEventListener('pointercancel', () => { this.scrubbing = false })
        this.range.addEventListener('pointerup', () => this.range.blur())
        this.audio.events.on('muteChange', () => this.paint())
        const initial = Math.floor(Date.now() / 180000) % songs.length
        this.select(initial, autoplay)
        setInterval(() => {
            if (!document.hidden && !this.scrubbing) this.paintPosition()
        }, 250)
    }

    select(index, play) {
        this.sound?.unload()
        this.index = (index + this.songs.length) % this.songs.length
        this.track = this.songs[this.index]
        this.wantsPlay = play
        this.scrubbing = false
        this.error = ''
        this.pendingCommand = null
        this.requestedPosition = null
        this.id = undefined
        this.title.textContent = this.track.name.replace(/\.mp3$/i, '')
        this.title.title = this.title.textContent
        const failed = message => {
            if (this.sound !== sound) return
            this.error = message
            this.wantsPlay = false
            this.paint()
        }
        const settled = command => {
            if (this.sound !== sound) return
            if (this.pendingCommand === command) this.pendingCommand = null
            this.synchronize()
            this.paint()
        }
        const sound = new Howl({
            src: [this.track.path],
            format: this.track.format ? [this.track.format] : undefined,
            html5: Boolean(this.track.format),
            volume: 0.2,
            preload: false,
            onload: () => {
                if (this.sound !== sound) return
                this.synchronize()
                this.paint()
            },
            onloaderror: () => failed('载入失败'),
            onplayerror: () => failed('播放失败'),
            onplay: () => settled('play'),
            onpause: () => settled('pause'),
            onseek: () => settled('seek'),
            onend: () => { if (this.sound === sound && this.wantsPlay) this.next(true) },
        })
        this.sound = sound
        sound.load()
        this.paint()
    }

    synchronize() {
        // HTMLAudioElement.play() is asynchronous. Sending Howler another
        // command before its play event can strand that command in its queue.
        // Seek a paused voice, then honor the most recent playback intent.
        if (this.sound.state() !== 'loaded' || this.error || this.pendingCommand) return
        const playing = this.sound.playing(this.id)
        const shouldPlay = this.wantsPlay && !this.suspended
        if (playing && (!shouldPlay || this.requestedPosition !== null)) {
            this.pendingCommand = 'pause'
            this.sound.pause(this.id)
        } else if (this.requestedPosition !== null) {
            const position = this.requestedPosition
            this.requestedPosition = null
            this.pendingCommand = 'seek'
            if (this.id === undefined) this.sound.seek(position)
            else this.sound.seek(position, this.id)
        } else if (!playing && shouldPlay) {
            this.pendingCommand = 'play'
            this.id = this.sound.play(this.id)
        }
    }

    setSuspended(value) {
        this.suspended = value
        this.synchronize()
        this.paint()
    }

    toggle() {
        if (this.error) {
            this.select(this.index, true)
            return
        }
        this.wantsPlay = !this.wantsPlay
        this.synchronize()
        this.paint()
    }

    next(play = this.wantsPlay || Boolean(this.error)) {
        this.audio.groups.get('discChange')?.play()
        this.select(this.index + 1, play)
    }

    previous() {
        this.audio.groups.get('discChange')?.play()
        this.select(this.index - 1, this.wantsPlay || Boolean(this.error))
    }

    seekTo(seconds) {
        const total = this.trackDuration()
        if (!Number.isFinite(seconds) || total === 0) return
        const position = Math.max(0, Math.min(seconds, total))
        this.requestedPosition = position
        this.synchronize()
        this.paintTime(position, total)
    }

    seekBy(seconds) {
        this.seekTo((this.requestedPosition ?? this.position()) + seconds)
    }

    trackDuration() {
        const duration = this.sound.duration()
        return Number.isFinite(duration) && duration > 0 ? duration : 0
    }

    position() {
        const position = this.sound.seek()
        return Number.isFinite(position) ? position : 0
    }

    paintPosition() {
        this.paintTime(this.position(), this.trackDuration())
    }

    paintTime(position, duration) {
        this.range.disabled = duration === 0 || Boolean(this.error)
        this.range.max = duration
        this.range.value = position
        const progress = duration ? Math.min(position / duration * 100, 100) : 0
        this.range.style.setProperty('--radio-progress', progress + '%')
        const elapsed = clock(position)
        const length = duration ? clock(duration) : '–:––'
        if (elapsed !== this.elapsed.textContent || length !== this.duration.textContent) {
            this.elapsed.textContent = elapsed
            this.duration.textContent = length
            this.range.setAttribute('aria-valuetext', elapsed + ' / ' + length)
        }
    }

    paint() {
        let status = ''
        if (this.error) status = this.error
        else if (this.audio.mute.active) status = '已静音'
        else if (this.sound.state() !== 'loaded') status = '调谐中'
        else if (!this.wantsPlay || this.suspended) status = '已暂停'
        if (this.status.textContent !== status) this.status.textContent = status
        const label = this.error ? '重试播放' : this.wantsPlay ? '暂停音乐' : '播放音乐'
        this.toggleButton.setAttribute('aria-label', label)
        this.toggleButton.title = label
        this.toggleButton.dataset.pause = this.wantsPlay
        if (!this.scrubbing) this.paintPosition()
    }
}
