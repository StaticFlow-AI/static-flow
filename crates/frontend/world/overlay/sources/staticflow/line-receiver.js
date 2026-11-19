// Roadside lab instruments. Each housing is a fixed Rapier body. Switching
// world lines only changes its powered indicators, so the road stays consistent.
import {
    BoxGeometry, CanvasTexture, Color, CylinderGeometry, Group, Mesh, MeshBasicMaterial,
    MeshStandardMaterial, OctahedronGeometry, PlaneGeometry, Quaternion, SRGBColorSpace, TorusGeometry, Vector3,
} from 'three/webgpu'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'

const palettes = {
    alpha: { housing: '#7a9287', signal: '#74ffe5' },
    beta: { housing: '#a5794f', signal: '#ffad43' },
    gate: { housing: '#9b9986', signal: '#fff1ba' },
}

function faceplate(kind) {
    const canvas = document.createElement('canvas')
    canvas.width = 384
    canvas.height = 160
    const ctx = canvas.getContext('2d')
    ctx.fillStyle = '#14272a'
    ctx.fillRect(0, 0, 384, 160)
    ctx.fillStyle = palettes[kind].signal
    ctx.font = 'bold 42px monospace'
    ctx.fillText({ alpha: 'IBN 5100', beta: 'FM  β', gate: 'FG LAB  TX' }[kind], 16, 49)
    if (kind === 'alpha') {
        for (const [row, text] of ['> LOAD ARCHIVE', '  0000: 31 41'].entries())
            ctx.fillText(text, 16, 96 + row * 38, 340)
    } else {
        ctx.lineWidth = 4
        ctx.strokeStyle = palettes[kind].signal
        ctx.beginPath()
        for (let x = 12; x < 374; x += 3) {
            const y = 104 + Math.sin(x * .12) * Math.cos(x * .033) * 37
            if (x === 12) ctx.moveTo(x, y)
            else ctx.lineTo(x, y)
        }
        ctx.stroke()
    }
    const map = new CanvasTexture(canvas)
    map.colorSpace = SRGBColorSpace
    const mat = new MeshStandardMaterial({ map })
    mat.name = 'line-receiver-display-' + kind
    return mat
}

function makeTransmission(ticker, kind) {
    const parts = []
    const material = new MeshBasicMaterial({
        color: palettes[kind].signal, transparent: true, depthWrite: false, toneMapped: false,
    })
    let animate

    if (kind === 'alpha') {
        // Broken scan lines on two intersecting planes read from both approach roads.
        for (let row = 0; row < 4; row++) {
            for (let col = 0; col < 5; col++) {
                if ((row * 2 + col) % 4 === 0) continue
                const x = (col - 2) * .37
                const y = row * .21
                parts.push(new BoxGeometry(.28, .058, .055).translate(x, y, 0))
                if (col !== 2) parts.push(new BoxGeometry(.055, .058, .28).translate(0, y, x))
            }
        }
        animate = () => {
            const sweep = .5 - .5 * Math.cos(ticker.elapsed * 2.8)
            mesh.position.y = 2.55 + .75 * sweep
            material.opacity = .70 + .28 * sweep
        }
    } else if (kind === 'beta') {
        // A short train of radio wavefronts grows away from the Yagi array.
        for (let i = 0; i < 3; i++) {
            const wave = new TorusGeometry(.39 + i * .28, .046, 6, 36)
            parts.push(wave.rotateX(Math.PI / 2).translate(0, i * .23, 0))
        }
        animate = () => {
            const phase = (ticker.elapsed % 2.3) / 2.3
            mesh.position.set(-.15, 3.55 + .50 * phase, -.21)
            mesh.scale.setScalar(.74 + .55 * phase)
            material.opacity = .55 + .41 * Math.sin(phase * Math.PI)
        }
    } else {
        // Paired streaks draw the eye into a bright focus above the two field coils.
        parts.push(new OctahedronGeometry(.30))
        for (const [dx, dy, dz] of [[1,0,0], [0,1,0], [0,0,1]]) {
            const ray = new OctahedronGeometry(1)
            parts.push(ray.scale(.075 + dx*.88, .075 + dy*.81, .075 + dz*.88))
            for (const sign of [-1, 1]) {
                const spark = new OctahedronGeometry(.092)
                parts.push(spark.translate(sign * dx, sign * dy, sign * dz))
            }
        }
        animate = () => {
            const contraction = (.5 + .5 * Math.cos(ticker.elapsed * 2.6)) ** 2
            mesh.position.y = 3.91
            mesh.rotation.y = ticker.elapsed * .46
            mesh.scale.setScalar(1.16 - .47 * contraction)
            material.opacity = .69 + .31 * contraction
        }
    }
    const mesh = new Mesh(mergeGeometries(parts), material)
    mesh.name = 'transmission-' + kind
    mesh.visible = false
    for (const geometry of parts) geometry.dispose()
    return { mesh, animate }
}

export function createLineReceiver(game, position, kind) {
    const palette = palettes[kind]
    const model = new Group()
    model.name = 'line-receiver-' + kind
    const groups = new Map()
    const colliders = []
    const paint = (label, hex) => {
        const mat = new MeshStandardMaterial({ color: hex })
        mat.name = 'line-receiver-' + label
        return mat
    }
    const housing = paint(kind, palette.housing)
    const iron = paint('iron', '#3e5453')
    const ink = paint('rubber', '#222c31')
    const enamel = paint('enamel', '#d3c49c')
    const brass = paint('brass', '#b78b56')
    const lamp = new MeshBasicMaterial()
    lamp.name = 'line-receiver-signal-' + kind
    lamp.userData.prevent = true
    const light = new Color(palette.signal).multiplyScalar(1.65)
    const dark = new Color('#33413f')
    lamp.color.copy(dark)

    // Bake directly to model-local coordinates so reel spokes, cooling fins
    // and antenna elements share one draw per surface finish.
    function piece(geometry, mat, x, y, z, rotationX = 0, rotationZ = 0) {
        if (rotationX) geometry.rotateX(rotationX)
        if (rotationZ) geometry.rotateZ(rotationZ)
        geometry.translate(x, y, z)
        if (!groups.has(mat)) groups.set(mat, [])
        groups.get(mat).push(geometry)
    }
    function box(mat, x, y, z, w, h, d, tilt = 0) {
        piece(new BoxGeometry(w, h, d), mat, x, y, z, tilt)
    }
    function cylinder(mat, x, y, z, r, h, face = false) {
        piece(new CylinderGeometry(r, r, h, 16), mat, x, y, z, face ? Math.PI / 2 : 0)
    }
    function rod(mat, a, b, radius) {
        const from = new Vector3(...a), to = new Vector3(...b)
        const direction = new Vector3().subVectors(to, from)
        const geometry = new CylinderGeometry(radius, radius, direction.length(), 8)
        geometry.applyQuaternion(new Quaternion().setFromUnitVectors(new Vector3(0, 1, 0), direction.normalize()))
        from.lerp(to, .5)
        piece(geometry, mat, from.x, from.y, from.z)
    }
    function solid(x, y, z, w, h, d) {
        colliders.push({ shape: 'cuboid', position: { x, y, z }, parameters: [w / 2, h / 2, d / 2] })
    }
    function display(x, y, z, w, h) {
        piece(new PlaneGeometry(w, h), faceplate(kind), x, y, z)
    }

    box(iron, 0, .10, 0, 2.06, .20, 1.41)
    solid(0, .10, 0, 2.06, .20, 1.41)

    if (kind === 'alpha') {
        // Twin exposed tape spools and a forward-sloping microcomputer keyboard.
        box(housing, 0, 1.02, -.16, 1.71, 1.65, .93)
        box(ink, 0, 1.47, .332, 1.53, .63, .08)
        for (const x of [-.39, .39]) {
            cylinder(brass, x, 1.48, .395, .257, .055, true)
            cylinder(ink, x, 1.48, .428, .18, .032, true)
            cylinder(enamel, x, 1.48, .452, .055, .022, true)
            for (let i = 0; i < 3; i++) {
                const angle = i * Math.PI * 2 / 3
                rod(enamel, [x,1.48,.45], [x+.19*Math.cos(angle),1.48+.19*Math.sin(angle),.45], .025)
            }
        }
        box(ink, -.23, .96, .34, .80, .36, .08)
        display(-.23, .96, .382, .71, .285)
        box(enamel, 0, .70, .39, 1.48, .13, .53, -.12)
        for (let row = 0; row < 3; row++)
            for (let col = 0; col < 9; col++)
                box(ink, -.55+col*.105, .779+row*.012, .34+row*.093, .075, .028, .064)
        box(lamp, .55, 1.03, .32, .22, .075, .040)
        cylinder(iron, .53, 1.89, -.19, .17, .085)
        cylinder(lamp, .53, 2.035, -.19, .125, .22)
        cylinder(brass, .53, 2.17, -.19, .142, .055)
        solid(0, 1.02, -.16, 1.71, 1.65, .93)
        solid(0, .69, .44, 1.48, .16, .50)
    } else if (kind === 'beta') {
        // Bench CRT with side channel selectors and a crossed Yagi antenna.
        box(iron, 0, .69, -.04, 1.49, 1.0, 1.05)
        box(housing, 0, 1.52, -.04, 1.77, .94, 1.15)
        box(enamel, -.20, 1.53, .55, 1.03, .65, .10)
        box(ink, -.20, 1.53, .606, .92, .53, .04)
        display(-.20, 1.53, .629, .85, .455)
        for (const y of [1.40, 1.68]) cylinder(ink, .58, y, .57, .115, .12, true)
        for (let i = 0; i < 6; i++) box(ink, .892, 1.31+i*.065, -.17, .028, .025, .59)
        cylinder(brass, -.48, 2.60, -.30, .046, 1.26)
        rod(iron, [-1.00,2.97,-.57], [.63,3.52,-.05], .035)
        for (let i = 0; i < 4; i++) {
            const x = -.88+i*.41, y = 3.01+i*.138, z = -.53+i*.131
            rod(enamel, [x-.12,y,z-.31], [x+.12,y,z+.31], .032)
        }
        cylinder(lamp, .58, 1.81, .57, .055, .09, true)
        cylinder(iron, .49, 2.03, -.24, .175, .08)
        cylinder(lamp, .49, 2.18, -.24, .13, .22)
        cylinder(brass, .49, 2.32, -.24, .15, .06)
        solid(0, .69, -.04, 1.49, 1, 1.05)
        solid(0, 1.52, -.04, 1.77, .94, 1.15)
        solid(-.48, 2.6, -.3, .11, 1.25, .11)
    } else {
        // Two field coils converge on the central transmit head.
        box(housing, 0, .80, 0, 1.78, 1.2, 1.14)
        box(enamel, 0, 1.45, 0, 1.92, .14, 1.22)
        box(ink, 0, .95, .59, 1.29, .49, .075)
        display(0, .95, .631, .98, .35)
        for (const sign of [-1, 1]) {
            const x = sign * .61
            cylinder(iron, x, 1.94, -.075, .16, .89)
            for (let i = 0; i < 5; i++) cylinder(brass, x, 1.66+i*.135, -.075, .219, .064)
            rod(iron, [x,2.36,-.075], [sign*.24,2.71,-.075], .076)
            cylinder(lamp, x, 2.36, -.075, .115, .09)
            solid(x, 1.93, -.075, .34, .85, .34)
        }
        piece(new TorusGeometry(.245, .064, 8, 24), enamel, 0, 2.72, -.075)
        cylinder(lamp, 0, 2.72, -.075, .125, .105, true)
        solid(0, .86, 0, 1.82, 1.32, 1.16)
    }
    for (const [material, geometries] of groups) {
        model.add(new Mesh(mergeGeometries(geometries), material))
        for (const geometry of geometries) geometry.dispose()
    }
    game.objects.add({ model }, { type: 'fixed', position, colliders, friction: .72 })
    const { mesh, animate } = makeTransmission(game.ticker, kind)
    model.add(mesh)
    return {
        setVisible(on) {
            if (mesh.visible === on) return
            mesh.visible = on
            lamp.color.copy(on ? light : dark)
            if (on) {
                animate()
                game.ticker.events.on('tick', animate)
            } else {
                game.ticker.events.off('tick', animate)
            }
        },
    }
}
