const buttonKey = 'HUD.drivingView'
const stickKey = 'Gamepad.tapR3'

function press(inputs, key) {
    inputs.start(key)
    inputs.end(key)
}

export function bindDrivingView(game) {
    const button = document.querySelector('[data-driving-view]')
    const { inputs } = game
    inputs.addActions([{ name: 'drivingView', categories: ['wandering', 'racing'], keys: ['Keyboard.KeyC', buttonKey, stickKey] }])
    inputs.events.on('drivingView', action => {
        if (!action.active || document.querySelector('dialog[open]')) return
        const camera = game.view.firstPerson
        camera.setEnabled(!camera.enabled)
        button.setAttribute('aria-pressed', String(camera.enabled))
    })
    // Keep a held accelerator pressed when the driver taps the HUD.
    button.addEventListener('pointerdown', event => {
        if (event.button === 0) event.preventDefault()
    })
    button.addEventListener('click', event => {
        press(inputs, buttonKey)
        if (event.detail) inputs.keyboard.focusDriving()
    })
}

// A tap changes perspective. Holding the stick keeps the original zoom control.
export function bindViewStick(inputs, zoom) {
    let holdTimer = null
    inputs.events.on('zoomToggle', action => {
        if (action.active) {
            holdTimer = setTimeout(() => {
                holdTimer = null
                zoom(action)
            }, 300)
        } else if (holdTimer !== null) {
            clearTimeout(holdTimer)
            holdTimer = null
            press(inputs, stickKey)
        } else {
            zoom(action)
        }
    })
}
