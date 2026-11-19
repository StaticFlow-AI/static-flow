import * as THREE from 'three/webgpu'

function canvas(width, height) {
    const element = document.createElement('canvas')
    element.width = width
    element.height = height
    return element.getContext('2d')
}

function texture(context) {
    const result = new THREE.CanvasTexture(context.canvas)
    result.colorSpace = THREE.SRGBColorSpace
    result.flipY = false
    result.generateMipmaps = false
    result.minFilter = result.magFilter = THREE.LinearFilter
    return result
}

// The career wall shader reads red as the illuminated portion of its stone inscription.
export function inscription(text) {
    const ctx = canvas(640, 120)
    ctx.fillStyle = '#000'
    ctx.fillRect(0, 0, 640, 120)
    ctx.fillStyle = '#fff'
    ctx.font = 'bold 68px sans-serif'
    ctx.textBaseline = 'middle'
    ctx.textAlign = 'center'
    ctx.fillText(text, 320, 60, 620)
    return texture(ctx)
}

function lines(ctx, text, width, maximum) {
    const rows = ['']
    for (const character of Array.from(text)) {
        const i = rows.length - 1
        if (ctx.measureText(rows[i] + character).width <= width) rows[i] += character
        else if (rows.length < maximum) rows.push(character)
        else { rows[i] = rows[i].slice(0, -1) + '…'; break }
    }
    return rows
}

function drawBoard(card, art) {
    const ctx = canvas(896, 504)
    ctx.fillStyle = card.vinyl ? '#173e40' : '#f3e7cb'
    ctx.fillRect(0, 0, 896, 504)
    const ink = card.vinyl ? '#fcdd9e' : '#30272d'
    if (art) {
        const side = Math.min(art.width, art.height)
        ctx.drawImage(art, (art.width - side) / 2, (art.height - side) / 2, side, side, 536, 88, 320, 320)
    } else if (card.vinyl) {
        for (let radius = 155; radius > 44; radius -= 5) {
            ctx.beginPath()
            ctx.arc(688, 250, radius, 0, 2 * Math.PI)
            ctx.fillStyle = '#19292c'
            ctx.strokeStyle = '#344b4d'
            ctx.fill()
            ctx.stroke()
        }
        ctx.beginPath()
        ctx.arc(688, 250, 44, 0, 2 * Math.PI)
        ctx.fillStyle = '#e37f53'
        ctx.fill()
    }
    ctx.fillStyle = ink
    ctx.font = 'bold 24px sans-serif'
    ctx.fillText(card.section, 40, 56, 810)
    ctx.fillRect(40, 75, 816, 3)
    const width = art || card.vinyl ? 466 : 806
    ctx.font = card.excerpt ? '32px sans-serif' : 'bold 51px sans-serif'
    for (const [index, line] of lines(ctx, card.excerpt || card.name, width, card.excerpt ? 6 : 4).entries())
        ctx.fillText(line, 40, 156 + index * (card.excerpt ? 45 : 65))
    ctx.fillRect(40, 419, 816, 3)
    ctx.font = '26px sans-serif'
    ctx.fillText(card.byline, 40, 466, 810)
    return texture(ctx)
}

const textures = new WeakMap()

export function loadBoard(card, ready) {
    // Lab thumbnails and their projection screen share an immutable texture.
    if (!textures.has(card)) {
        textures.set(card, (async () => {
            let art
            if (card.art) {
                try {
                    const response = await fetch(card.art, { signal: AbortSignal.timeout(4000) })
                    if (response.ok) art = await createImageBitmap(await response.blob())
                } catch {}
            }
            const result = drawBoard(card, art)
            art?.close()
            return result
        })())
    }
    textures.get(card).then(ready)
}

// Keep each prop's original Rapier body, mass, orientation and collision shape.
// The new signs inherit the moving bodies when the vehicle knocks them off the plinth.
export function dressDirectory(area, links) {
    const models = ['x', 'bluesky', 'youtube', 'mail', 'twitch', 'github', 'linkedin', 'discord', 'onlyfans']
    const titles = [...links.map(item => item.name), '风扇']
    const colors = ['#873e28', '#174d4a', '#58416a', '#495335']
    const hidden = new THREE.MeshBasicMaterial({ visible: false })
    for (const object of area.objects.items) {
        const base = object?.visual?.object3D
        const slot = base ? models.indexOf(base.name.toLowerCase().replace(/[_\d.]+$/, '')) : -1
        if (slot < 0) continue
        const bounds = new THREE.Box3().setFromObject(base, true)
        const size = bounds.getSize(new THREE.Vector3())
        const center = base.worldToLocal(bounds.getCenter(new THREE.Vector3()))
        base.traverse(child => {
            if (child.isMesh) child.material = hidden
        })
        const ctx = canvas(256, 256)
        ctx.fillStyle = colors[slot % colors.length]
        ctx.fillRect(0, 0, 256, 256)
        ctx.strokeStyle = '#e8cea0'
        ctx.lineWidth = 3
        ctx.strokeRect(12, 12, 232, 232)
        ctx.fillStyle = '#f5e2bc'
        ctx.textAlign = 'center'
        ctx.textBaseline = 'middle'
        ctx.font = 'bold 64px sans-serif'
        const title = titles[slot]
        if (title.length > 2) {
            ctx.fillText(title.slice(0, 2), 128, 92)
            ctx.fillText(title.slice(2), 128, 166)
        } else ctx.fillText(title, 128, 128)
        const faceTexture = texture(ctx)
        faceTexture.flipY = true
        const face = new THREE.MeshLambertMaterial({ map: faceTexture })
        const edge = new THREE.MeshLambertMaterial({ color: '#a27d59' })
        const block = new THREE.Mesh(new THREE.BoxGeometry(size.x, size.y, size.z), [edge, edge, edge, edge, face, face])
        block.position.copy(center)
        block.quaternion.copy(base.quaternion).invert()
        block.castShadow = block.receiveShadow = true
        base.add(block)
    }
}
