import { BufferAttribute, Color, Mesh } from 'three/webgpu'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'

export function paintPart(parts, geometry, color) {
    const g = geometry.index ? geometry.toNonIndexed() : geometry
    g.deleteAttribute('uv')
    g.computeVertexNormals()
    const n = g.getAttribute('normal')
    const colors = new Float32Array(n.count * 3)
    const base = new Color(color)
    for (let i = 0; i < n.count; i++) {
        // Broad, cool windscreen light across the console. The interior is
        // legible even when the outside sun passes behind the vehicle.
        const light = .63 + .39 * Math.max(0, n.getX(i) * -.36 + n.getY(i) * .67 + n.getZ(i) * .65)
        colors[i * 3] = base.r * light
        colors[i * 3 + 1] = base.g * light
        colors[i * 3 + 2] = base.b * light
    }
    g.setAttribute('color', new BufferAttribute(colors, 3))
    g.deleteAttribute('normal')
    if (g !== geometry) geometry.dispose()
    parts.push(g)
}

export function assemble(parts, material) {
    const geometry = mergeGeometries(parts)
    for (const piece of parts) piece.dispose()
    return new Mesh(geometry, material)
}
