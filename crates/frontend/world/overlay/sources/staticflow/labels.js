import { CanvasTexture } from 'three/webgpu'
import { texture, uv } from 'three/tsl'

// Coordinates are the text rectangles in the pinned area's two UV atlases.
// The original controller glyphs occupy the right half of the blackboard.
export function localizedLabelColor(material) {
    const board = material.name === 'blackboardLabels'
    const cells = board
        ? [['下一项', 0, 5, 114, 41], ['上一项', 0, 56, 115, 41], ['打开', 0, 106, 117, 41],
           ['退出', 0, 156, 104, 41], ['滚动', 0, 205, 125, 42]]
        : [['笔记', 12, 4, 195, 23], ['日期', 7, 33, 69, 22], ['作者', 90, 33, 70, 22], ['分类', 172, 33, 32, 22]]
    const width = board ? 255 : 218
    const height = board ? 256 : 60
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
    return board ? uv().x.greaterThan(127 / width).select(texture(material.map).rgb, localized) : localized
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
