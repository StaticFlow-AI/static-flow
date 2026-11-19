import assert from 'node:assert/strict'
import test from 'node:test'
import { chromium } from 'playwright'

const worldUrl = process.env.STATICFLOW_WORLD_URL

test('driving keeps the keyboard after using fullscreen, the minimap and bindings', {
    skip: !worldUrl && 'Set STATICFLOW_WORLD_URL to a running world preview',
    timeout: 120_000,
}, async () => {
    const browser = await chromium.launch({
        channel: process.env.PLAYWRIGHT_CHANNEL,
        args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
    })
    try {
        const page = await browser.newPage()
        await page.addInitScript(() => {
            if (window !== window.top) return
            // Media shortcut handlers can register before the game loads.
            window.mediaKeys = []
            for (const type of ['keydown', 'keyup']) {
                window.addEventListener(type, event => {
                    if (/^Key[WASDI]$/.test(event.code) && !event.target.closest?.('input, textarea, [contenteditable]'))
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
    } finally {
        await browser.close()
    }
})
