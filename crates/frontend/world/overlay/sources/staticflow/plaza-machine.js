// A miniature Radio Kaikan roof with the FG204 capsule lodged in its deck.
import * as THREE from 'three/webgpu'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'

function paint(name, color) {
    const material = new THREE.MeshStandardMaterial({ color })
    material.name = 'fg204-' + name
    return material
}

function lettering(name, width, height, draw) {
    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    draw(canvas.getContext('2d'))
    const texture = new THREE.CanvasTexture(canvas)
    texture.colorSpace = THREE.SRGBColorSpace
    const material = new THREE.MeshBasicMaterial({ map: texture })
    material.name = 'fg204-' + name
    return material
}

function part(group, geometry, material, x, y, z) {
    const mesh = new THREE.Mesh(geometry, material)
    mesh.position.set(x, y, z)
    group.add(mesh)
    return mesh
}

function box(group, material, x, y, z, width, height, depth) {
    return part(group, new THREE.BoxGeometry(width, height, depth), material, x, y, z)
}

function ring(group, material, y, bottomRadius, topRadius, height) {
    return part(group, new THREE.CylinderGeometry(topRadius, bottomRadius, height, 20), material, 0, y, 0)
}

// Objects.getFromModel consumes named direct children as Rapier descriptions.
function collider(group, shape, position, size, rotationZ = 0) {
    const marker = new THREE.Object3D()
    marker.name = shape
    marker.position.set(...position)
    marker.scale.set(...size)
    marker.rotation.z = rotationZ
    group.add(marker)
}

// Static details on the same body and with the same paint share a draw call.
function batchSurfaces(group) {
    const buckets = new Map()
    for (const mesh of [...group.children]) {
        if (!mesh.isMesh) continue
        mesh.updateMatrix()
        const geometry = mesh.geometry.applyMatrix4(mesh.matrix)
        if (!buckets.has(mesh.material)) buckets.set(mesh.material, [])
        buckets.get(mesh.material).push(geometry)
        mesh.removeFromParent()
    }
    for (const [material, geometries] of buckets) {
        const merged = mergeGeometries(geometries)
        group.add(new THREE.Mesh(merged, material))
        for (const geometry of geometries) geometry.dispose()
    }
}

function panelTexture() {
    return lettering('cells', 192, 256, ctx => {
        ctx.fillStyle = '#292c3c'
        ctx.fillRect(0, 0, 192, 256)
        for (let row = 0; row < 7; row++) {
            for (let col = 0; col < 5; col++) {
                const x = 4 + col * 37, y = 4 + row * 36
                ctx.fillStyle = ['#485874', '#536581', '#4e587e'][(row + col) % 3]
                ctx.fillRect(x, y, 34, 32)
                ctx.fillStyle = '#91a0b0'
                ctx.fillRect(x + 16, y, 1, 32)
            }
        }
    })
}

function makeCapsule() {
    const pod = new THREE.Group()
    pod.name = 'refStatuePhysicalDynamic'
    pod.userData.mass = 3.4
    pod.userData.friction = 0.55

    const ivory = paint('blanket', '#e9dfc8')
    const rim = paint('burnt-metal', '#494737')
    const silver = paint('brushed-alloy', '#afa99b')
    const vermilion = paint('vermilion', '#b74730')
    const dark = paint('heatshield', '#303038')
    ring(pod, dark, -1.19, .58, .81, .48)
    ring(pod, rim, -.82, .84, 1.06, .28)
    ring(pod, dark, -.50, 1.07, 1.11, .36)
    for (const y of [-.65, -.55, -.45]) ring(pod, silver, y, 1.094, 1.094, .025)
    ring(pod, vermilion, -.28, 1.13, 1.15, .15)
    ring(pod, ivory, .25, 1.15, 1.08, .93)
    ring(pod, silver, .23, 1.148, 1.148, .055)
    // Narrow battens split the insulated drum into quilted panels.
    for (let i = 0; i < 16; i++) {
        const a = i * Math.PI / 8
        const seam = part(pod, new THREE.CylinderGeometry(.022, .028, .92, 6),
            silver, Math.sin(a) * 1.115, .25, Math.cos(a) * 1.115)
        seam.rotation.set(-Math.cos(a) * .075, 0, Math.sin(a) * .075)
    }
    ring(pod, vermilion, .78, 1.10, 1.018, .14)
    ring(pod, silver, .99, 1.015, .63, .31)
    ring(pod, rim, 1.22, .64, .55, .17)
    ring(pod, ivory, 1.34, .55, .52, .09)

    const sensor = part(pod, new THREE.SphereGeometry(.24, 12, 8), ivory, -.17, 1.52, .015)
    sensor.scale.z = .86
    part(pod, new THREE.CylinderGeometry(.042, .06, .73, 8), silver, .20, 1.66, -.04)
    part(pod, new THREE.CylinderGeometry(.025, .035, .40, 8), ivory, .32, 1.53, -.06)
    const stencil = lettering('designation', 320, 120, ctx => {
        ctx.fillStyle = '#e5dec8'
        ctx.fillRect(0, 0, 320, 120)
        ctx.fillStyle = '#39372e'
        ctx.textAlign = 'center'
        ctx.font = 'bold 63px monospace'
        ctx.fillText('FG204', 160, 68)
        ctx.font = 'bold 20px monospace'
        ctx.fillText('2ND EDITION', 160, 98)
    })
    part(pod, new THREE.PlaneGeometry(.80, .30), stencil, 0, .47, 1.14)
    const light = new THREE.MeshBasicMaterial({ color: '#ffc563' })
    light.name = 'fg204-signal'
    for (let i = 0; i < 3; i++) box(pod, light, -.16 + i * .16, -.49, 1.10, .085, .07, .032)

    // The photovoltaic wing wraps around the port side on three metal hinges.
    const cells = panelTexture()
    box(pod, silver, -1.07, -.11, 0, .64, .15, .20)
    let x = -1.37, y = -.91
    for (const bend of [.26, -.18, -.79]) {
        const height = .99, width = .95, depth = .115
        const cx = x - Math.sin(bend) * height / 2
        const cy = y + Math.cos(bend) * height / 2
        box(pod, rim, cx, cy, 0, depth, height, width).rotation.z = bend
        for (const side of [-1, 1]) {
            const face = part(pod, new THREE.PlaneGeometry(width - .07, height - .09), cells,
                cx + side * Math.cos(bend) * .061, cy + side * Math.sin(bend) * .061, 0)
            face.rotation.order = 'ZXY'
            face.rotation.set(0, side * Math.PI / 2, bend)
        }
        collider(pod, 'cuboid', [cx, cy, 0], [depth, height, width], bend)
        x -= Math.sin(bend) * height
        y += Math.cos(bend) * height
        const pin = part(pod, new THREE.CylinderGeometry(.05, .05, 1.0, 8), silver, x, y, 0)
        pin.rotation.x = Math.PI / 2
    }

    collider(pod, 'tube', [0, .02, 0], [2.22, 1.71, 2.22])
    collider(pod, 'tube', [0, -1.11, 0], [1.27, .64, 1.27])
    collider(pod, 'tube', [0, 1.05, 0], [1.38, .60, 1.38])
    batchSurfaces(pod)
    return pod
}

function makeRoof() {
    const roof = new THREE.Group()
    roof.name = 'radioKaikanPhysicalFixed'
    const concrete = paint('concrete', '#9e9680')
    const parapet = paint('parapet', '#66655c')
    const enamel = paint('trim', '#e4c58e')
    box(roof, concrete, 0, .49, 0, 3.68, .98, 3.39)
    box(roof, parapet, 0, 1.00, 0, 3.76, .12, 3.47)
    collider(roof, 'cuboid', [0, .53, 0], [3.76, 1.06, 3.47])
    // Raised rooftop plant room with the old building's sun-faded tiled glass.
    box(roof, concrete, 0, 1.76, -1.37, 3.48, 1.4, .60)
    box(roof, enamel, 0, 2.49, -1.37, 3.56, .10, .68)
    collider(roof, 'cuboid', [0, 1.80, -1.37], [3.55, 1.48, .68])
    const glass = ['#4c929b', '#d8a25c', '#bc7977', '#a99c76'].map((c, i) => paint('tile' + i, c))
    for (let i = 0; i < 8; i++) {
        for (let row = 0; row < 2; row++) {
            box(roof, glass[(i * 3 + row) % 4], (i - 3.5) * .423, 1.43 + .60 * row, -1.04, .39, .56, .055)
        }
    }
    const sign = lettering('radio-kaikan', 672, 128, ctx => {
        ctx.fillStyle = '#2d2923'; ctx.fillRect(0, 0, 672, 128)
        ctx.fillStyle = '#e9bf79'; ctx.fillRect(5, 5, 662, 118)
        ctx.fillStyle = '#28251f'
        ctx.textAlign = 'center'
        ctx.font = 'bold 70px sans-serif'
        ctx.fillText('ラジオ会館', 336, 86)
    })
    part(roof, new THREE.PlaneGeometry(2.86, .55), sign, 0, .56, 1.706)
    batchSurfaces(roof)
    return roof
}

export function installPlazaMachine(model) {
    const original = model.children.find(node => node.name.startsWith('refStatuePhysicalDynamic'))
    const base = model.children.find(node => node.name.startsWith('physicalFixed'))
    const oldFooting = base?.children.find(node => node.name === 'hull')
    if (!original || !oldFooting) throw new Error('Missing social plaza statue or footing')
    const roof = makeRoof()
    roof.position.copy(base.position)
    oldFooting.removeFromParent()
    for (const node of [...model.children]) {
        if (node === original || /^(boy|sudo|baguira)PhysicalDynamic/.test(node.name)) node.removeFromParent()
    }
    const pod = makeCapsule()
    pod.position.copy(roof.position).add(new THREE.Vector3(.08, 2.56, .20))
    pod.rotation.set(-.07, .52, -.13)
    model.add(roof, pod)
}
