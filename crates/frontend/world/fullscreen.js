// Embedded beside the launch controls so fullscreen runs in the original
// gesture, even if the renderer and the blog catalog are still loading.
(() => {
    const root = document.documentElement
    const launch = document.getElementById('world-fullscreen')
    const launchLabel = launch.querySelector('span')
    const drive = document.getElementById('world-drive')
    const toggle = document.querySelector('[data-world-fullscreen]')
    const hint = document.querySelector('.screen-hint')
    const feedback = document.querySelectorAll('.screen-feedback')
    const enter = root.requestFullscreen || root.webkitRequestFullscreen
    const leave = document.exitFullscreen || document.webkitExitFullscreen
    const supported = Boolean(enter && leave && (document.fullscreenEnabled ?? document.webkitFullscreenEnabled ?? true))
    const active = () => Boolean(document.fullscreenElement || document.webkitFullscreenElement)
    const returnToDriving = () => window.dispatchEvent(new Event('world-driving-focus'))
    let startWhenReady = false
    let pending = false

    function report(text) {
        for (const node of feedback) {
            node.textContent = text
            node.hidden = !text
        }
    }

    function render() {
        const full = active()
        const title = full ? '退出全屏' : '进入全屏'
        toggle.hidden = !supported
        toggle.disabled = pending
        toggle.setAttribute('aria-label', title)
        toggle.title = title + ' (Alt + Enter)'
        toggle.setAttribute('aria-pressed', String(full))
        launch.disabled = !supported || pending || (full && startWhenReady)
        launchLabel.textContent = full && startWhenReady ? '已全屏，载入后出发' : '全屏驾驶'
    }

    function startIfReady() {
        if (!startWhenReady || !active() || drive.hidden || root.classList.contains('world-entered')) return
        startWhenReady = false
        drive.click()
    }

    function denied() {
        startWhenReady = false
        report(root.classList.contains('world-entered') ? '浏览器未允许切换全屏。' : '浏览器未允许全屏，可选择「开始驾驶」使用窗口模式。')
        render()
    }

    async function change(fromLaunch = false) {
        if (pending || !supported || root.classList.contains('world-failed')) return
        const full = active()
        startWhenReady = !root.classList.contains('world-entered') && (!full || fromLaunch)
        report('')
        if (full && fromLaunch) { startIfReady(); render(); return }
        pending = true
        try {
            if (full) await leave.call(document)
            else await enter.call(root)
            startIfReady()
        } catch {
            denied()
        } finally {
            pending = false
            render()
        }
    }

    function changed() {
        if (!active()) startWhenReady = false
        report('')
        render()
        startIfReady()
        if (root.classList.contains('world-entered')) returnToDriving()
    }

    launch.addEventListener('click', () => change(true))
    drive.addEventListener('click', () => {
        startWhenReady = false
        report('')
    })
    // Keep a pointer press on the HUD from taking the next driving key.
    // Fullscreen transitions may finish several frames after the click.
    toggle.addEventListener('pointerdown', event => {
        if (event.button === 0) event.preventDefault()
    })
    toggle.addEventListener('click', event => {
        const transition = change()
        if (event.detail) {
            returnToDriving()
            transition.then(returnToDriving)
        }
    })
    window.addEventListener('world-drive-ready', startIfReady)
    window.addEventListener('world-fullscreen-shortcut', () => change())
    window.addEventListener('keydown', event => {
        if (event.key !== 'Enter' || !event.altKey || event.ctrlKey || event.metaKey || event.shiftKey || event.isComposing) return
        if (document.querySelector('dialog[open]') || event.target.closest?.('input,textarea,select,[contenteditable]:not([contenteditable=false])')) return
        if (!supported) return
        event.preventDefault()
        event.stopImmediatePropagation()
        if (!event.repeat) change()
    }, true)
    document.addEventListener('fullscreenchange', changed)
    document.addEventListener('webkitfullscreenchange', changed)
    document.addEventListener('webkitfullscreenerror', denied)
    if (!supported) {
        hint.textContent = '当前浏览器不支持网页全屏，可使用窗口模式驾驶。'
    }
    render()
})()
