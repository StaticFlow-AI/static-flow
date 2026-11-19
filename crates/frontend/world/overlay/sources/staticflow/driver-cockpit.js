// Camera-local interior. The dial, hands and sleeves are separate moving
// assemblies; static pieces of each assembly share one vertex-coloured mesh.
import {
    BoxGeometry, BufferAttribute, BufferGeometry, Color, CylinderGeometry,
    ExtrudeGeometry, Group, MathUtils, Mesh, MeshBasicMaterial, Shape,
    SphereGeometry, TorusGeometry, Vector3,
} from 'three/webgpu'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'

const up = new Vector3(0, 1, 0)
const wheelRadius = .170
const gripAngle = Math.PI / 8

function part(parts, geometry, color) {
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

function assemble(parts, material) {
    const geometry = mergeGeometries(parts)
    for (const piece of parts) piece.dispose()
    return new Mesh(geometry, material)
}

function box(parts, color, x, y, z, w, h, depth, tilt = 0) {
    part(parts, new BoxGeometry(w, h, depth).rotateZ(tilt).translate(x, y, z), color)
}

function plate(parts, color, outline, depth, x, y, z, angle = 0) {
    const shape = new Shape()
    shape.moveTo(...outline[0])
    for (const point of outline.slice(1)) shape.lineTo(...point)
    shape.closePath()
    const geometry = new ExtrudeGeometry(shape, { depth, bevelEnabled: false })
    part(parts, geometry.rotateZ(angle).translate(x, y, z), color)
}

function disc(parts, color, x, y, z, radius, depth, sides = 32) {
    part(parts, new CylinderGeometry(radius, radius, depth, sides)
        .rotateX(Math.PI / 2).translate(x, y, z), color)
}

function oval(parts, color, x, y, z, rx, ry, rz, angle = 0) {
    part(parts, new SphereGeometry(1, 10, 6).scale(rx, ry, rz).rotateZ(angle).translate(x, y, z), color)
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
    part(parts, geometry, color)
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
        part(parts, new CylinderGeometry(upper, lower, end - start, 9).translate(0, (start + end) / 2, 0), tone)
    }
    return assemble(parts, material)
}

function wheelMesh(material) {
    const parts = []
    part(parts, new TorusGeometry(wheelRadius, .015, 8, 48), '#383632')
    const spoke = [[-.016,.022],[.038,.023],[.142,.012],[.158,.005],[.150,-.013],[.053,-.009],[.020,-.023]]
    for (const rotation of [0, Math.PI, -Math.PI / 2]) {
        plate(parts, '#676b69', spoke, .010, 0, 0, -.008, rotation)
        const r = .095
        box(parts, '#303331', r * Math.cos(rotation), r * Math.sin(rotation), .003, .053, .005, .002, rotation)
    }
    disc(parts, '#302d2a', 0, 0, .010, .048, .028, 12)
    disc(parts, '#866b49', 0, 0, .026, .031, .005, 12)
    disc(parts, '#292b2b', 0, 0, .030, .026, .005, 12)
    for (const angle of [-.49, .49]) {
        box(parts, '#deb579', .004 * Math.sign(angle), 0, .034, .003, .017, .002, angle)
    }
    // Brass twelve-o'clock band makes even a slight turn easy to read.
    part(parts, new TorusGeometry(wheelRadius, .016, 8, 4, .080)
        .rotateZ(Math.PI / 2 - .040), '#d8a65f')
    return assemble(parts, material)
}

function dashboard(material) {
    const parts = []
    const outline = [[-3,-.52],[3,-.52],[3,-.012],[.34,.016],[.23,.045],[-.20,.045],[-.30,.018],[-3,-.012]]
    plate(parts, '#49443e', outline, .15, 0, -.415, -1.01)
    // The dash cap slopes away to the windscreen seal. Its bright top and
    // shadowed front edge give the road a fixed frame at the driver's eye.
    const cap = new BufferGeometry()
    const sections = [[-3,-.368,-.404],[-.45,-.354,-.393],[-.24,-.354,-.372],
        [.24,-.354,-.372],[.45,-.354,-.393],[3,-.368,-.404]]
    const corners = []
    for (const [x, front, back] of sections) corners.push(x, front, -1.11, x, back, -.855)
    cap.setAttribute('position', new BufferAttribute(new Float32Array(corners), 3))
    const faces = []
    for (let i = 0; i < sections.length - 1; i++) {
        const n = i * 2
        faces.push(n, n + 1, n + 2, n + 1, n + 3, n + 2)
    }
    cap.setIndex(faces)
    part(parts, cap, '#655c51')
    box(parts, '#373735', 0, -.355, -1.115, 3, .009, .018)
    box(parts, '#7e6150', 0, -.429, -.855, 5, .008, .006)
    // A raised binnacle behind the wheel, a mechanical steering indicator
    // on the left, and a bank of louvers on the passenger side.
    const binnacle = [[-.168,-.113],[.175,-.113],[.168,-.016],[.125,.033],[-.113,.033],[-.162,-.011]]
    plate(parts, '#252a2c', binnacle, .040, .025, -.281, -.915)
    disc(parts, '#8f795b', .025, -.299, -.874, .049, .007)
    disc(parts, '#151d20', .025, -.299, -.867, .045, .005)
    for (let i = 0; i < 13; i++) {
        const a = Math.PI * (i - 6) / 9
        const r = .036
        const long = i % 3 === 0
        box(parts, long ? '#ffcc7e' : '#ac9274', .025 + Math.sin(a) * r,
            -.299 + Math.cos(a) * r, -.861, long ? .0025 : .0016, long ? .008 : .005, .002, -a)
    }
    box(parts, '#8ebcb5', -.077, -.294, -.873, .019, .007, .003)
    box(parts, '#c39a58', -.049, -.294, -.873, .013, .007, .003)
    for (const x of [-.35, .36]) {
        box(parts, '#242729', x, -.401, -.850, .12, .035, .008)
        for (const y of [-.009, 0, .009])
            box(parts, '#645f53', x, -.401 + y, -.842, .103, .0025, .002)
    }
    return assemble(parts, material)
}

export function createDriverCockpit() {
    const material = new MeshBasicMaterial({ vertexColors: true, fog: false })
    const group = new Group()
    group.name = 'driver-cockpit'
    group.add(dashboard(material))

    const needle = new Group()
    needle.position.set(.025, -.299, -.856)
    const parts = []
    box(parts, '#f4ad50', 0, .014, 0, .0035, .029, .003)
    disc(parts, '#a48966', 0, 0, .002, .006, .003)
    needle.add(assemble(parts, material))
    group.add(needle)

    const column = new Group()
    column.position.set(0, -.308, -.745)
    column.rotation.x = -.21
    group.add(column)
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
    return { group, update(steeringAngle, aspect, fov) {
        // Keep the eye-to-wheel proportions across different vertical FOVs;
        // on tall screens pull the assembly in far enough to see both hands.
        const lens = Math.tan(MathUtils.degToRad(fov) / 2) / Math.tan(MathUtils.degToRad(68) / 2)
        const fit = Math.min(1, aspect / .62)
        group.scale.set(.45 * lens * fit, .45 * lens * fit, .45)
        group.position.y = -.45 * lens * .36 * (1 - fit)
        const angle = MathUtils.clamp(steeringAngle * 2.15, -1.22, 1.22)
        if (angle === lastAngle) return
        lastAngle = angle
        wheel.rotation.z = angle
        needle.rotation.z = angle
        for (const { arm, wrist } of arms) {
            end.copy(wrist).applyAxisAngle(axis, angle).sub(arm.position)
            arm.scale.y = end.length()
            arm.quaternion.setFromUnitVectors(up, end.normalize())
        }
    } }
}
