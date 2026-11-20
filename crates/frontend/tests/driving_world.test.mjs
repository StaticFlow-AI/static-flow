import assert from 'node:assert/strict'
import test from 'node:test'
import { chromium, devices } from 'playwright'

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

test('a compatibility-only phone draws the driving world without HDR color targets', {
    skip: !worldUrl && 'Set STATICFLOW_WORLD_URL to a running world preview',
    timeout: 120_000,
}, async t => {
    const browser = await chromium.launch({
        channel: process.env.PLAYWRIGHT_CHANNEL,
        args: ['--enable-unsafe-webgpu', '--force-webgpu-compat', '--use-webgpu-adapter=opengles',
            '--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
    })
    try {
        const page = await browser.newPage({ ...devices['Pixel 7'], viewport: { width: 400, height: 792 } })
        const framebufferErrors = []
        page.on('console', message => {
            if (message.text().includes('GL_INVALID_FRAMEBUFFER_OPERATION')) framebufferErrors.push(message.text())
        })
        await page.addInitScript(() => {
            const prototype = WebGL2RenderingContext.prototype
            const getExtension = prototype.getExtension
            const getSupported = prototype.getSupportedExtensions
            const hdr = new Set(['EXT_color_buffer_float', 'EXT_color_buffer_half_float'])
            prototype.getExtension = function (name) {
                return hdr.has(name) ? null : getExtension.call(this, name)
            }
            prototype.getSupportedExtensions = function () {
                return getSupported.call(this)?.filter(name => !hdr.has(name)) ?? null
            }
        })
        await page.goto(worldUrl, { waitUntil: 'domcontentloaded' })
        const compatibilityOnly = await page.evaluate(async () => {
            if (!navigator.gpu) return false
            const compat = await navigator.gpu.requestAdapter({ featureLevel: 'compatibility' })
            const core = await navigator.gpu.requestAdapter({ featureLevel: 'core', powerPreference: 'high-performance' })
            return compat && !core?.features.has('core-features-and-limits')
        })
        if (!compatibilityOnly) return t.skip('This Chromium build does not expose a compatibility-only GPU adapter')
        // The world requests shared state when the opening animation finishes.
        await Promise.all([page.waitForRequest('**/api/world', { timeout: 90_000 }),
            page.locator('#world-drive').tap({ timeout: 90_000 })])
        assert.ok(await page.locator('canvas.experience').evaluate(canvas => !!canvas.getContext('webgl2')))

        // Sample the main road, excluding DOM controls. A sky gradient alone
        // can span many colors; edges distinguish actual geometry from fog.
        const shot = await page.screenshot({ clip: { x: 40, y: 250, width: 320, height: 330 }, scale: 'css' })
        const edges = await page.evaluate(async png => {
            const image = new Image()
            image.src = 'data:image/png;base64,' + png
            await image.decode()
            const ctx = new OffscreenCanvas(96, 96).getContext('2d')
            ctx.drawImage(image, 0, 0, 96, 96)
            const { data } = ctx.getImageData(0, 0, 96, 96)
            let edges = 0
            for (let i = 4; i < data.length; i += 4) {
                if ((i / 4) % 96 === 0) continue
                if (Math.max(...[0, 1, 2].map(c => Math.abs(data[i + c] - data[i + c - 4]))) > 16) edges++
            }
            return edges
        }, shot.toString('base64'))
        assert.ok(edges > 96 * 95 * .025, `expected visible terrain and car, found ${edges} edges`)
        assert.deepEqual(framebufferErrors, [])
    } finally {
        await browser.close()
    }
})
