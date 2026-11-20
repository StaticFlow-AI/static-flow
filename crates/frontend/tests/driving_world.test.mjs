import assert from 'node:assert/strict'
import test from 'node:test'
import { chromium } from 'playwright'

const worldUrl = process.env.STATICFLOW_WORLD_URL

test('driving controls steer the cockpit and keep focus across the HUD', {
    skip: !worldUrl && 'Set STATICFLOW_WORLD_URL to a running world preview',
    timeout: 120_000,
}, async () => {
    const browser = await chromium.launch({
        channel: process.env.PLAYWRIGHT_CHANNEL,
        args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
    })
    try {
        const page = await browser.newPage()
        const errors = []
        page.on('pageerror', error => errors.push(error.message))
        await page.addInitScript(() => {
            if (window !== window.top) return
            // Three's inspector hook observes the same scene the visitor sees.
            window.worldScenes = []
            window.__THREE_DEVTOOLS__ = new EventTarget()
            window.__THREE_DEVTOOLS__.addEventListener('observe', ({ detail }) => {
                if (detail.isScene) window.worldScenes.push(detail)
            })
            // Media shortcut handlers can register before the game loads.
            window.mediaKeys = []
            for (const type of ['keydown', 'keyup']) {
                window.addEventListener(type, event => {
                    if (/^Key[WASDIC]$/.test(event.code) && !event.target.closest?.('input, textarea, [contenteditable]'))
                        window.mediaKeys.push([type, event.code])
                }, true)
            }
        })
        await page.goto(worldUrl, { waitUntil: 'domcontentloaded' })
        await page.locator('#world-fullscreen').click()
        await page.waitForFunction(() => document.fullscreenElement && document.documentElement.classList.contains('input-filter-wandering'), null, { timeout: 90_000 })

        const screen = page.locator('[data-world-fullscreen]')
        await screen.click()
        await page.keyboard.press('d')
        await page.waitForFunction(() => !document.fullscreenElement)

        const view = page.getByRole('button', { name: '第一人称驾驶' })
        assert.equal(await view.getAttribute('aria-pressed'), 'false')
        await page.keyboard.down('w')
        await page.keyboard.down('c')
        await page.keyboard.down('c')
        await page.keyboard.up('c')
        assert.equal(await view.getAttribute('aria-pressed'), 'true')
        await page.keyboard.up('w')
        await page.waitForFunction(() => window.worldScenes.some(scene => scene.getObjectByName('driver-cockpit')?.visible))
        const cockpit = await page.evaluateHandle(() =>
            window.worldScenes.map(scene => scene.getObjectByName('driver-cockpit')).find(group => group?.visible))
        await page.waitForFunction(group => {
            const mirror = group.getObjectByName('rear-window-reflection')
            return mirror?.visible && mirror.material.map?.isRenderTargetTexture
        }, cockpit)
        const gripHeights = () => cockpit.evaluate(group => ['grip-left', 'grip-right'].map(name => {
            const hand = group.getObjectByName(name)
            return group.worldToLocal(hand.getWorldPosition(hand.position.clone())).y
        }))
        await page.keyboard.down('a')
        await page.waitForFunction(group => group.getObjectByName('steering-wheel').rotation.z > .2, cockpit)
        const turningLeft = await gripHeights()
        await page.keyboard.up('a')
        await page.keyboard.down('d')
        await page.waitForFunction(group => group.getObjectByName('steering-wheel').rotation.z < -.2, cockpit)
        const turningRight = await gripHeights()
        assert.ok(turningLeft[0] < turningRight[0] && turningLeft[1] > turningRight[1])
        await page.keyboard.up('d')
        await page.waitForFunction(group => Math.abs(group.getObjectByName('steering-wheel').rotation.z) < .03, cockpit)
        await page.keyboard.down('w')
        await page.waitForFunction(group =>
            group.getObjectByName('speed-readout').userData.speedKmh > 2 &&
            group.getObjectByName('gear-display').userData.drive === 'D' &&
            group.getObjectByName('throttle-indicator').scale.x > 0, cockpit)
        await page.keyboard.down('b')
        await page.waitForFunction(group => group.getObjectByName('brake-indicator').scale.x > 0, cockpit)
        await page.keyboard.up('b')
        await view.click()
        assert.equal(await view.getAttribute('aria-pressed'), 'false')
        await page.waitForFunction(group => !group.visible, cockpit)
        await page.keyboard.up('w')

        const minimap = page.locator('[data-minimap-toggle]')
        assert.equal(await minimap.getAttribute('aria-expanded'), 'true')
        await minimap.click()
        await page.keyboard.press('a')
        assert.equal(await minimap.getAttribute('aria-expanded'), 'false')
        await minimap.click()
        await page.keyboard.press('s')

        await page.locator('#world-key-settings').click()
        await page.locator('[data-binding=forward]').click()
        await page.keyboard.press('i')
        await page.locator('.world-bindings [data-close]').click()
        // A physical press immediately after the dialog closes must reach the
        // vehicle. The live map follows it using the same world coordinates.
        await page.keyboard.down('i')
        const terrain = page.locator('[data-minimap-terrain]')
        const start = await terrain.evaluate(image => image.style.transform)
        await page.waitForFunction(start => document.querySelector('[data-minimap-terrain]').style.transform !== start, start)
        await page.keyboard.up('i')
        assert.deepEqual(await page.evaluate(() => window.mediaKeys), [])
        assert.deepEqual(errors, [])
    } finally {
        await browser.close()
    }
})
