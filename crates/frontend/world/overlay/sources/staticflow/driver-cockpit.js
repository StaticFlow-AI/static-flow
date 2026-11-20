// The wheel and gloves rotate about the steering column. The elbows stay at
// the seat while each sleeve tracks its wrist.
import {
    BoxGeometry, BufferAttribute, BufferGeometry, Color, CylinderGeometry,
    ExtrudeGeometry, Group, MathUtils, MeshBasicMaterial, Path, Shape,
    SphereGeometry, TorusGeometry, Vector3,
} from 'three/webgpu'
import { createCockpitCabin } from './cockpit-cabin.js'
import { createCockpitInstruments } from './cockpit-instruments.js'
import { assemble, paintPart } from './cockpit-mesh.js'

const up = new Vector3(0, 1, 0)
const wheelRadius = .170
const gripAngle = Math.PI / 8

function box(parts, color, x, y, z, w, h, depth, tilt = 0) {
    paintPart(parts, new BoxGeometry(w, h, depth).rotateZ(tilt).translate(x, y, z), color)
}

function disc(parts, color, x, y, z, radius, depth, sides = 32) {
    paintPart(parts, new CylinderGeometry(radius, radius, depth, sides)
        .rotateX(Math.PI / 2).translate(x, y, z), color)
}

function oval(parts, color, x, y, z, rx, ry, rz, angle = 0) {
    paintPart(parts, new SphereGeometry(1, 10, 6).scale(rx, ry, rz).rotateZ(angle).translate(x, y, z), color)
}

// Sweep a tapered polygon around the centerline of a curved finger, so the
// knuckles wrap the outside of the grip and the pad ends behind the rim.
function finger(parts, color, path) {
    const positions = []
    const indices = []
    const sides = 7
    const vertex = new Vector3()
    const tangent = new Vector3()
    const across = new Vector3()
    const outward = new Vector3()
    const points = path.map(([x, y, z]) => new Vector3(x, y, z))
    for (let ring = 0; ring < path.length; ring++) {
        tangent.subVectors(points[Math.min(ring + 1, path.length - 1)], points[Math.max(0, ring - 1)]).normalize()
        across.set(0, 1, 0)
        if (Math.abs(tangent.y) > .92) across.set(0, 0, 1)
        across.addScaledVector(tangent, -across.dot(tangent)).normalize()
        outward.crossVectors(tangent, across)
        for (let i = 0; i < sides; i++) {
            const a = 2 * Math.PI * i / sides
            vertex.copy(points[ring]).addScaledVector(across, path[ring][3] * Math.cos(a))
                .addScaledVector(outward, path[ring][3] * Math.sin(a))
            positions.push(vertex.x, vertex.y, vertex.z)
            if (ring > 0) {
                const p = (ring - 1) * sides + i
                const q = (ring - 1) * sides + (i + 1) % sides
                indices.push(p, q, p + sides, q, q + sides, p + sides)
            }
        }
    }
    const geometry = new BufferGeometry()
    geometry.setAttribute('position', new BufferAttribute(new Float32Array(positions), 3))
    geometry.setIndex(indices)
    paintPart(parts, geometry, color)
}

function handMesh(material) {
    const parts = []
    const skin = '#cd8e6c'
    const knuckle = '#e4aa83'
    oval(parts, skin, .016, -.001, .022, .026, .044, .016, -.18)
    oval(parts, skin, .031, -.042, .037, .0235, .044, .020, .38)
    for (const [y, length, r] of [[.030, 1, .009], [.012, 1.04, .0093], [-.006, .94, .0086], [-.022, .77, .0078]]) {
        finger(parts, knuckle, [
            [.012, y, .030, r * .9], [.031 * length, y + .002, .036, r],
            [.044 * length, y + .001, .021, r], [.041 * length, y, .001, r * .88],
            [.027 * length, y, -.012, r * .79], [.010, y + .001, -.015, r * .48],
        ])
    }
    finger(parts, '#eeb38c', [
        [.018, -.021, .030, .014], [.001, -.010, .038, .013],
        [-.011, .008, .035, .012], [-.014, .022, .025, .010],
        [-.013, .029, .017, .0045],
    ])
    oval(parts, '#ecc1a8', -.012, .020, .031, .0058, .008, .003, -.12)
    // Fingerless driving gloves expose the fingertips around the back of the
    // rim. The inset knuckle openings catch light against the dark leather.
    oval(parts, '#413630', .019, -.004, .030, .021, .040, .014, -.18)
    oval(parts, '#493d35', .030, -.041, .041, .021, .033, .017, .38)
    for (const [x, y] of [[.022,.022],[.026,.009],[.027,-.005],[.026,-.018]]) {
        oval(parts, '#211f20', x, y, .043, .0048, .0042, .002)
        oval(parts, skin, x, y, .045, .0034, .0030, .0016)
    }
    oval(parts, '#282b2c', .039, -.061, .050, .023, .013, .014, .38)
    disc(parts, '#ae9070', .033, -.058, .064, .0043, .002, 10)
    return assemble(parts, material)
}

function sleeveMesh(material) {
    const parts = []
    // The sleeve's local Y runs from elbow to wrist. The taper and the
    // contrasting knit undercuff stay attached as the forearm changes angle.
    const bands = [
        [0, .056, .59, .046, '#c3c6bc'],
        [.59, .046, .66, .048, '#a4aaa4'],
        [.66, .048, .85, .035, '#d7d8ca'],
        [.85, .040, .95, .035, '#e3dfcc'],
        [.95, .030, 1, .026, '#313b3c'],
    ]
    for (const [start, lower, end, upper, tone] of bands) {
        paintPart(parts, new CylinderGeometry(upper, lower, end - start, 9).translate(0, (start + end) / 2, 0), tone)
    }
    return assemble(parts, material)
}

function wheelMesh(material) {
    const parts = []
    paintPart(parts, new TorusGeometry(wheelRadius, .015, 8, 48), '#383632')
    // Stamped aluminium spokes are dished back from the grip. Actual holes
    // leave little glimpses of the cluster as the driver turns the wheel.
    const spoke = new Shape()
    spoke.moveTo(.023, .021)
    spoke.quadraticCurveTo(.095, .020, .159, .012)
    spoke.lineTo(.159, -.011)
    spoke.quadraticCurveTo(.076, -.011, .026, -.025)
    spoke.closePath()
    for (const [x, radius] of [[.070, .009], [.101, .0075], [.128, .006]]) {
        const hole = new Path()
        hole.absarc(x, .0015, radius, 0, Math.PI * 2, true)
        spoke.holes.push(hole)
    }
    for (const rotation of [.12, Math.PI - .12, -Math.PI / 2]) {
        const g = new ExtrudeGeometry(spoke, { depth: .005, bevelEnabled: true,
            bevelSegments: 1, bevelSize: .0014, bevelThickness: .0014, curveSegments: 6 })
        paintPart(parts, g.rotateY(-.085).translate(0, 0, -.014).rotateZ(rotation), '#8d9794')
    }
    disc(parts, '#282b2c', 0, 0, -.012, .047, .037)
    disc(parts, '#697478', 0, 0, .003, .043, .010)
    disc(parts, '#23272a', 0, 0, .011, .032, .009)
    disc(parts, '#4a4940', 0, 0, .018, .025, .008)
    for (let i = 0; i < 6; i++) {
        const a = i * Math.PI / 3 + Math.PI / 6
        const x = Math.cos(a) * .037, y = Math.sin(a) * .037
        disc(parts, '#a4a79e', x, y, .010, .0039, .003, 8)
        disc(parts, '#343434', x, y, .013, .0018, .0008, 6)
    }
    for (const angle of [-.49, .49])
        box(parts, '#e9c68d', .004 * Math.sign(angle), 0, .024, .003, .018, .002, angle)

    // Paired leather grips, a stitched inside seam and the woven centre mark.
    for (const angle of [-.57, Math.PI - .57])
        paintPart(parts, new TorusGeometry(wheelRadius, .019, 10, 14, 1.14).rotateZ(angle), '#312e2c')
    for (let i = 0; i < 56; i++) {
        const a = i * Math.PI / 28
        box(parts, '#8e7360', .164 * Math.cos(a), .164 * Math.sin(a), .0135,
            .0042, .0009, .0014, a + 1.20)
    }
    paintPart(parts, new TorusGeometry(wheelRadius, .016, 10, 5, .085)
        .rotateZ(Math.PI / 2 - .0425), '#efad55')
    return assemble(parts, material)
}

function steeringColumn(material) {
    const parts = []
    disc(parts, '#24292c', 0, 0, -.082, .041, .15, 20)
    for (const z of [-.058, -.047, -.036])
        disc(parts, '#414749', 0, 0, z, .044, .006, 20)
    for (const side of [-1, 1]) {
        box(parts, '#434a48', side * .095, .006, -.055, .131, .008, .010, side * .14)
        oval(parts, '#2d3336', side * .157, .016, -.054, .034, .009, .012, side * .14)
        for (const x of [.175, .180])
            box(parts, '#807e6f', side * x, .020, -.043, .0018, .009, .001, side * .14)
    }
    return assemble(parts, material)
}

export function createDriverCockpit() {
    const material = new MeshBasicMaterial({ vertexColors: true, fog: false })
    const group = new Group()
    group.name = 'driver-cockpit'
    const cabin = createCockpitCabin()
    const instruments = createCockpitInstruments()
    const controls = new Group()
    group.add(cabin.group, controls)
    controls.add(instruments.group)
    const column = new Group()
    column.position.set(0, -.308, -.745)
    column.rotation.x = -.21
    controls.add(column)
    column.add(steeringColumn(material))
    const wheel = new Group()
    wheel.name = 'steering-wheel'
    wheel.add(wheelMesh(material))
    column.add(wheel)

    const hand = handMesh(material)
    const sleeve = sleeveMesh(material)
    const arms = [-1, 1].map(side => {
        const grip = new Group()
        grip.name = side === 1 ? 'grip-right' : 'grip-left'
        grip.position.set(side * wheelRadius * Math.cos(gripAngle), wheelRadius * Math.sin(gripAngle), 0)
        grip.rotation.z = side * gripAngle
        grip.scale.x = side
        grip.add(hand.clone())
        wheel.add(grip)

        const arm = sleeve.clone()
        arm.name = side === 1 ? 'sleeve-right' : 'sleeve-left'
        arm.position.set(side * .285, -.260, .22)
        column.add(arm)
        grip.updateMatrix()
        const wrist = new Vector3(.040, -.068, .046).applyMatrix4(grip.matrix)
        return { arm, wrist }
    })
    const axis = new Vector3(0, 0, 1)
    const end = new Vector3()
    let lastAngle
    const white = new Color(0xffffff)
    return { group, setRearViewTexture: cabin.setRearViewTexture, update(steeringAngle, aspect, fov, telemetry) {
        cabin.update(aspect, fov, telemetry)
        instruments.update(telemetry)
        material.color.lerpColors(white, telemetry.daylight, .48)
        cabin.shellMaterial.color.copy(material.color)
        // Keep the eye-to-wheel proportions across different vertical FOVs;
        // on tall screens pull the assembly in far enough to see both hands.
        const lens = Math.tan(MathUtils.degToRad(fov) / 2) / Math.tan(MathUtils.degToRad(68) / 2)
        const fit = Math.min(1, aspect / .62)
        controls.scale.set(.45 * lens * fit, .45 * lens * fit, .45)
        controls.position.y = -.45 * lens * .36 * (1 - fit)
        const angle = MathUtils.clamp(steeringAngle * 2.15, -1.22, 1.22)
        if (angle === lastAngle) return
        lastAngle = angle
        wheel.rotation.z = angle
        for (const { arm, wrist } of arms) {
            end.copy(wrist).applyAxisAngle(axis, angle).sub(arm.position)
            arm.scale.y = end.length()
            arm.quaternion.setFromUnitVectors(up, end.normalize())
        }
    } }
}
