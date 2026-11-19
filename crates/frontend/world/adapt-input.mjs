import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

export default async function adaptInput(root) {
    const files = [
        ['Game/Inputs/Inputs.js', [
            ['new Keyboard()', 'new Keyboard(this)\n        this.game.debug.bindKeyboard?.(this.keyboard)'],
            ["this.keyboard.events.on('down', (key, code) =>", "this.keyboard.events.on('down', key =>"],
            ["this.keyboard.events.on('up', (key, code) =>", "this.keyboard.events.on('up', key =>"],
            ['\n            this.start(`Keyboard.${code}`)', ''],
            ['\n            this.end(`Keyboard.${code}`)', ''],
            ['        this.setInteractiveButtons()', `        this.setInteractiveButtons()
        // Start, map and exhibit controls may be the visitor's first touch.
        document.addEventListener('pointerdown', event => {
            if (event.pointerType === 'touch') this.updateMode(Inputs.MODE_TOUCH)
        }, { capture: true, passive: true })`],
        ]],
        ['Game/Debug.js', [[
            "addEventListener('keydown', (event) =>",
            "this.bindKeyboard = keyboard => keyboard.events.on('sequence', (event) =>"
        ]]],
        ['Game/KonamiCode.js', [
            ["document.addEventListener('keydown', callback)", "this.game.inputs.keyboard.events.on('sequence', callback)"],
            ["document.removeEventListener('keydown', callback)", "this.game.inputs.keyboard.events.off('sequence', callback)"]
        ]],
        ['Game/InteractivePoints.js', [
            ["        this.game.inputs.events.on('interact', (action) =>", `        // A mouse hover can end while the car is parked, including as the
        // map fades after teleporting. Resolve the point at the car on input.
        const interact = () => {
            const hovered = this.activeItem
            this.needsTest = true
            this.update()
            if (!this.activeItem && hovered?.state === InteractivePoints.STATE_OPEN)
                this.activeItem = hovered
            if (this.activeItem?.state === InteractivePoints.STATE_OPEN)
                this.activeItem.interact()
        }
        this.game.inputs.events.on('interact', (action) =>`],
            ["if(action.active && this.activeItem && this.activeItem.state === InteractivePoints.STATE_OPEN)\n            {\n                this.activeItem.interact()\n            }", "if(action.active) interact()"],
            ["if(this.activeItem && this.activeItem.state === InteractivePoints.STATE_OPEN)\n            {\n                this.activeItem.interact()\n            }", "interact()"],
            ["                    item.conceal()\n                    this.activeItem = null", "                    item.conceal()\n                    this.activeItem = null\n                    this.needsTest = true"],
        ]],
        ['index.html', [
            ['</head>', '<link rel="stylesheet" href="./staticflow/shortcuts.css">\n</head>'],
            ['<span class="key">WASD</span>', '<span class="key" data-driving-keys>W A S D</span>'],
            ['<div class="js-content content controls-content">', '<div class="js-content content controls-content"><button type="button" class="binding-launch" data-open-bindings>设置快捷键</button>']
        ]]
    ]
    for (const [file, edits] of files) {
        const dest = join(root, 'sources', file)
        let source = await readFile(dest, 'utf8')
        for (const [before, after] of edits) {
            if (!source.includes(before)) throw new Error('Missing keyboard integration in ' + file)
            source = source.replace(before, after)
        }
        await writeFile(dest, source)
    }
}
