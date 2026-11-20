import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const indent = block => block.replace(/^([ \t]*\S)/gm, '    $1')

export default async function adaptWorldBuffers(root) {
    const edits = [
        ['Rendering.js', [
            ["import { Game } from './Game.js'", "import { Game } from './Game.js'\nimport { createWorldRenderer } from '../staticflow/world-renderer.js'"],
            [/        this\.renderer = new THREE\.WebGPURenderer\(\{\n[\s\S]*?^        \}\)/m,
                '        this.renderer = await createWorldRenderer(this.game.canvasElement, this.game.viewport.pixelRatio < 2)'],
            [/^        this\.bloomPass = bloom\(scenePassColor\)[\s\S]*?^        this\.bloomPass\.smoothWidth\.value = 1$/m,
                block => `        const hdr = this.renderer.getOutputBufferType() !== THREE.UnsignedByteType
        if (hdr) {
${indent(block)}
        }`],
            ['cheapDOF(renderOutput(scenePass))',
                'cheapDOF(hdr ? renderOutput(scenePass) : scenePassColor)'],
            [`            if(level === 0)
            {
                this.postProcessing.outputNode = this.cheapDOFPass.add(this.bloomPass)
            }
            else if(level === 1)
            {
                this.postProcessing.outputNode = scenePassColor.add(this.bloomPass)
            }`, `            const color = level === 0 ? this.cheapDOFPass : scenePassColor
            this.postProcessing.outputNode = hdr ? color.add(this.bloomPass) : color`],
            [/^            const bloomPanel = this\.debugPanel[\s\S]*?^            bloomPanel\.addBinding\(this\.bloomPass\.smoothWidth[^\n]*$/m,
                block => `            if (hdr) {
${indent(block)}
            }`],
        ]],
        ['Noises.js', [
            ['type: THREE.HalfFloatType', 'type: this.game.rendering.renderer.getOutputBufferType()'],
        ]],
        ['World/Snow.js', [
            ['        this.snowElevation = {}', `        const type = this.game.rendering.renderer.getOutputBufferType()
        this.snowElevation = { normalized: type === THREE.UnsignedByteType }`],
            ['type: THREE.HalfFloatType,', 'type,'],
            ['return vec4(elevation, 0, 0, 1)', `// Underwater terrain can lower the snow surface to -3 world units.
            const sample = this.snowElevation.normalized ? elevation.add(3).div(5) : elevation
            return vec4(sample, 0, 0, 1)`],
            ['const snowTextureElevation = texture(this.snowElevation.texture, newUv).r',
                `const sample = texture(this.snowElevation.texture, newUv).r
            const snowTextureElevation = this.snowElevation.normalized ? sample.mul(5).sub(3) : sample`],
        ]],
    ]
    for (const [file, replacements] of edits) {
        const path = join(root, 'sources/Game', file)
        let source = await readFile(path, 'utf8')
        for (const [before, after] of replacements) {
            const found = typeof before === 'string' ? source.includes(before) : before.test(source)
            if (!found) throw new Error('Missing render buffer integration in ' + file + ': ' + before)
            source = typeof before === 'string' ? source.replaceAll(before, after) : source.replace(before, after)
        }
        await writeFile(path, source)
    }
}
