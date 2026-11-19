import { CanvasTexture } from 'three/webgpu'
import { texture, uv } from 'three/tsl'

// The control blackboard keeps its controller glyphs on the right half of the UV atlas.
// The article workshop captions are embedded in the GLB during the build.
export function localizedLabelColor(material) {
    if (material.name === 'projectsLabels') return texture(material.map).rgb
    const cells = [['下一项', 0, 5, 114, 41], ['上一项', 0, 56, 115, 41], ['打开', 0, 106, 117, 41],
        ['退出', 0, 156, 104, 41], ['滚动', 0, 205, 125, 42]]
    const width = 255
    const height = 256
    const canvas = document.createElement('canvas')
    canvas.width = width * 3
    canvas.height = height * 3
    const context = canvas.getContext('2d')
    context.scale(3, 3)
    context.fillStyle = 'black'
    context.fillRect(0, 0, width, height)
    context.fillStyle = 'white'
    context.textAlign = 'center'
    context.textBaseline = 'middle'
    for (const [label, x, y, w, h] of cells) {
        context.font = 'bold ' + Math.floor(h * .89) + 'px sans-serif'
        context.fillText(label, x + w / 2, y + h / 2, w)
    }
    const atlas = new CanvasTexture(canvas)
    atlas.flipY = false
    const localized = texture(atlas).rgb
    return uv().x.greaterThan(127 / width).select(texture(material.map).rgb, localized)
}

export function drivePrompt(device) {
    const canvas = document.createElement('canvas')
    canvas.width = 512
    canvas.height = 256
    const context = canvas.getContext('2d')
    context.font = 'bold 70px sans-serif'
    context.textAlign = 'center'
    context.textBaseline = 'middle'
    context.fillStyle = 'white'
    const key = { mouseKeyboard: '回车', gamepadXbox: 'A', gamepadPlaystation: '×', touch: '轻点小车' }[device]
    context.fillText(key, 256, 118)
    const prompt = new CanvasTexture(canvas)
    prompt.flipY = false
    return prompt
}
