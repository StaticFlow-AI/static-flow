// Two laboratory appliances share enamel panels, bakelite controls and copper
// coils. Bake the rigid details into one draw per finish before Area registers
// the props with its normal materials, frustum and Rapier pipeline.
import {
    BoxGeometry, CanvasTexture, CatmullRomCurve3, CylinderGeometry, ExtrudeGeometry, Group,
    Mesh, MeshStandardMaterial, Object3D, PlaneGeometry, Shape, SRGBColorSpace,
    TorusGeometry, TubeGeometry, Vector3,
} from 'three/webgpu'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js'
import { color } from 'three/tsl'
import { MeshDefaultMaterial } from '../Game/Materials/MeshDefaultMaterial.js'

const finishes = {
    enamel: '#a9c2ae', cream: '#e8dbab', dark: '#263437', iron: '#596c68',
    copper: '#bb6b39', brass: '#e6a84d', red: '#9d432b', charcoal: '#131f22',
    gel: '#96df51', peel: '#a8ee58', hot: '#ff7d32',
}
const materials = new Map()

function paint(name) {
    if (!materials.has(name)) {
        const glowing = ['gel', 'hot'].includes(name)
        const finish = glowing ? new MeshDefaultMaterial({
            colorNode: color(finishes[name]).mul(1.45), hasCoreShadows: false, hasDropShadows: false,
            hasLightBounce: false, hasWater: false,
        }) : new MeshStandardMaterial({ color: finishes[name] })
        finish.userData.prevent = glowing
        finish.name = 'lab-hearth-' + name
        materials.set(name, finish)
    }
    return materials.get(name)
}

function part(parent, geometry, finish, x, y, z) {
    const mesh = new Mesh(geometry, typeof finish === 'string' ? paint(finish) : finish)
    mesh.position.set(x, y, z)
    parent.add(mesh)
    return mesh
}

function box(parent, w, h, d, x, y, z, finish, bevel = 0) {
    return part(parent, bevel ? new RoundedBoxGeometry(w, h, d, 1, bevel) : new BoxGeometry(w, h, d),
        finish, x, y, z)
}

function cylinder(parent, radius, height, x, y, z, finish) {
    return part(parent, new CylinderGeometry(radius, radius, height, 16), finish, x, y, z)
}

function cable(parent, points, radius, finish) {
    const curve = new CatmullRomCurve3(points.map(p => new Vector3(...p)))
    return part(parent, new TubeGeometry(curve, points.length * 3, radius, 6, false), finish, 0, 0, 0)
}

function plaque(parent, title, w, h, x, y, z, back, ink) {
    const canvas = document.createElement('canvas')
    canvas.width = 512
    canvas.height = Math.round(512 * h / w)
    const ctx = canvas.getContext('2d')
    ctx.fillStyle = back
    ctx.fillRect(0, 0, canvas.width, canvas.height)
    ctx.fillStyle = ink
    ctx.font = 'bold ' + Math.floor(canvas.height * .8) + 'px monospace'
    ctx.textBaseline = 'middle'
    ctx.textAlign = 'center'
    ctx.fillText(title, 256, canvas.height * .54, 486)
    const map = new CanvasTexture(canvas)
    map.colorSpace = SRGBColorSpace
    const mat = new MeshStandardMaterial({ map })
    mat.name = 'lab-lettering-' + title
    return part(parent, new PlaneGeometry(w, h), mat, x, y, z)
}

function bake(group, frame) {
    const byMaterial = new Map()
    for (const mesh of group.children) {
        mesh.updateMatrix()
        const geometry = mesh.geometry.index ? mesh.geometry.toNonIndexed() : mesh.geometry
        geometry.applyMatrix4(mesh.matrix)
        if (!byMaterial.has(mesh.material)) byMaterial.set(mesh.material, [])
        byMaterial.get(mesh.material).push(geometry)
        if (geometry !== mesh.geometry) mesh.geometry.dispose()
    }
    for (const [material, geometries] of byMaterial) {
        frame.add(new Mesh(mergeGeometries(geometries), material))
        for (const geometry of geometries) geometry.dispose()
    }
}

function collider(frame, w, h, d, x, y, z) {
    const marker = new Object3D()
    marker.name = 'cuboid'
    marker.position.set(x, y, z)
    marker.scale.set(w, h, d)
    frame.add(marker)
}

function node(root, name) {
    // GLTFLoader removes periods from Blender's numeric suffixes.
    const found = root.children.find(child => child.name.replaceAll('.', '') === name)
    if (!found) throw new Error('Missing oven scene node ' + name)
    return found
}

export function installPhoneWave(model) {
    const frame = new Group()
    frame.name = 'phoneWavePhysicalFixed'
    frame.position.copy(node(model, 'refBonfireHashes').position)
    for (const name of ['base015', 'sword', 'refBonfireBurn001', 'refBonfireHashes']) {
        model.remove(node(model, name))
    }
    frame.rotation.y = .40
    model.add(frame)
    const parts = new Group()

    // Salvaged workbench with a suspended cable shelf.
    box(parts, 3.62, .16, 1.9, 0, .96, 0, 'copper', .04)
    box(parts, 3.48, .15, 1.74, 0, .83, 0, 'iron')
    box(parts, 3.28, .09, 1.52, 0, .29, 0, 'dark')
    for (const x of [-1.52, 1.52]) for (const z of [-.68,.68]) {
        cylinder(parts, .075, .79, x, .46, z, 'iron')
        cylinder(parts, .13, .10, x, .08, z, 'charcoal')
    }
    collider(frame, 3.45, 1, 1.76, 0, .50, 0)

    // Recess the turntable and banana inside a deep enamel door frame.
    box(parts, 2.90, 1.45, 1.15, -.15, 1.78, -.17, 'enamel', .12)
    box(parts, 2.88, .16, .40, -.15, 1.16, .49, 'cream', .035)
    box(parts, 2.88, .16, .40, -.15, 2.42, .49, 'cream', .035)
    box(parts, .17, 1.24, .40, -1.49, 1.79, .49, 'cream', .03)
    box(parts, .75, 1.24, .40, .92, 1.79, .49, 'cream', .055)
    box(parts, 2.0, 1.05, .04, -.43, 1.80, .42, 'charcoal')
    cylinder(parts, .68, .045, -.42, 1.32, .48, 'iron').scale.z = .32
    const banana = new Shape()
    banana.moveTo(-1.03, 1.92)
    banana.bezierCurveTo(-.95, 1.62, -.50, 1.46, -.15, 1.57)
    banana.quadraticCurveTo(.04, 1.62, .17, 1.81)
    banana.bezierCurveTo(.09, 1.50, -.15, 1.37, -.46, 1.37)
    banana.bezierCurveTo(-.83, 1.37, -1.03, 1.59, -1.08, 1.88)
    banana.closePath()
    const peel = new ExtrudeGeometry(banana, { depth: .07, curveSegments: 9,
        bevelEnabled: true, bevelSize: .025, bevelThickness: .04, bevelSegments: 2 })
    part(parts, peel, 'peel', 0, 0, .46)
    cable(parts, [[-1.08,1.98,.50],[-1.05,1.88,.50]], .038, 'copper')
    for (let i=0; i<6; i++) box(parts,.60,.028,.044,.42,2.505,-.50+i*.075,'iron')
    for (const x of [-1.28,.43]) box(parts,.036,1.02,.06,x,1.81,.70,'dark')
    box(parts,.10,.90,.15,.47,1.80,.78,'iron',.04)
    plaque(parts,'00:08', .56,.255, .92,2.10,.70,'#1a2828','#9ee59c')
    for (const y of [1.47,1.78]) {
        cylinder(parts,.13,.09,.92,y,.72,'dark').rotation.x = Math.PI / 2
        box(parts,.022,.10,.014,.92,y+.025,.774,'cream')
    }
    collider(frame, 2.90, 1.45, 1.44, -.15, 1.77, -.02)

    // Copper antenna winding and the cabled flip phone that operates the oven.
    cylinder(parts,.23,.54,-.87,2.78,-.05,'iron')
    for(let i=0; i<5; i++) cylinder(parts,.27,.042,-.87,2.54+i*.105,-.05,'copper')
    cylinder(parts,.055,.36,-.87,3.21,-.05,'brass')
    box(parts,.48,.055,.50,.47,2.54,.14,'dark',.03)
    const lid = box(parts,.48,.49,.07,.47,2.78,-.05,'dark',.04)
    lid.rotation.x=-.20
    const screen = box(parts,.34,.33,.03,.47,2.81,.0,'gel',.015)
    screen.rotation.x=lid.rotation.x
    for (let row=0; row<3; row++) for (let col=0; col<3; col++)
        box(parts,.072,.024,.065,.355+col*.11,2.575,.10+row*.10,'cream')
    cable(parts,[[.64,2.55,.14],[.97,2.60,.07],[1.42,2.32,-.10],[1.49,1.66,-.08],[1.25,1.46,-.32]],.032,'charcoal')
    plaque(parts,'PHONEWAVE',1.36,.20,-.44,2.41,.702,'#263437','#eddfa9')
    bake(parts, frame)

    const vent = new Object3D()
    vent.name = 'refPhoneWaveVent'
    vent.position.set(-.87,3.39,-.05).applyQuaternion(frame.quaternion).add(frame.position)
    model.add(vent)
    const heater = part(frame, new TorusGeometry(.23,.032,6,20), 'hot', -.87,2.98,-.05)
    heater.rotation.x = Math.PI / 2
    heater.name = 'refPhoneWaveHeater'
}

export function installLaboratoryOven(model) {
    const frame = node(model,'cookiePhysicalFixed')
    const banner = node(frame,'refBanner'), panel = node(frame,'refCounterPanel'), heat = node(frame,'refOvenHeat')
    // Keep the game-owned animated nodes, replace the contiguous old stone
    // building, its lava floor, banner mast and original collision hulls.
    frame.clear()
    frame.add(banner, panel, heat)
    const parts = new Group()
    const spawner = node(model, 'refSpawner')
    frame.position.set(spawner.position.x, -model.position.y, spawner.position.z-1.55)
    box(parts,3.65,.27,2.65,0,.16,0,'iron',.07)
    box(parts,3.51,.54,2.41,0,.54,0,'dark',.07)
    box(parts,3.38,.16,2.44,0,.88,0,'copper',.035)
    box(parts,3.32,1.72,.74,0,1.62,-.80,'red',.12)
    for(const x of [-1.40,1.40]) {
        box(parts,.55,1.56,2.42,x,1.61,0,'enamel',.07)
        collider(frame,.57,1.6,2.42,x,1.60,0)
        for (let i=0;i<4;i++) box(parts,.36,.07,.05,x,1.10+i*.16,1.215,'iron')
    }
    box(parts,3.58,.45,2.54,0,2.51,0,'cream',.09)
    box(parts,3.69,.11,2.58,0,2.77,0,'copper',.035)
    collider(frame,3.58,.9,2.55,0,.45,0)
    collider(frame,3.50,.55,2.52,0,2.51,0)
    collider(frame,2.24,1.46,.74,0,1.59,-.80)

    // The heater sits inside an open tunnel; the cookie can leave through
    // its mouth with the impulse used by the original stand.
    box(parts,2.26,1.39,.12,0,1.57,-.39,'charcoal')
    for (let i=0;i<3;i++) box(parts,1.78,.066,.07,0,1.27+i*.22,-.31,'hot',.02)
    box(parts,2.23,.07,1.59,0,.99,.30,'iron')
    plaque(parts,'F.G. LAB',.88,.23,-1.10,2.54,1.277,'#243234','#fed079')
    panel.position.set(.30,2.53,1.282)
    panel.quaternion.identity()
    panel.scale.set(1,.63,1)
    const counter = node(model, 'refCounterLabel')
    counter.quaternion.identity()
    counter.position.copy(frame.position).add(new Vector3(.30,2.53,1.375))
    heat.position.set(0,1.25,-.25)
    spawner.position.copy(frame.position).add(new Vector3(0,1.39,1.50))

    // Banded exhaust and dial retain the station's readout at a distance.
    cylinder(parts,.39,1.03,-.62,3.22,-.61,'iron')
    for(const y of [2.93,3.28,3.65]) cylinder(parts,.435,.085,-.62,y,-.61,'copper')
    cylinder(parts,.35,.04,-.62,3.76,-.61,'charcoal')
    node(model,'refChimney').position.copy(frame.position).add(new Vector3(-.62,3.78,-.61))
    for (let i=0; i<6; i++) {
        box(parts,.055,.07,1.31,1.687,1.36+i*.13,-.17,'dark')
        box(parts,.85,.026,.045,.45,2.84,-.75+i*.12,'iron')
    }
    cylinder(parts,.32,.14,.98,2.97,.08,'dark').rotation.x=Math.PI/2
    cylinder(parts,.26,.06,.98,2.97,.18,'cream').rotation.x=Math.PI/2
    box(parts,.045,.27,.035,1.035,3.01,.23,'red').rotation.z=-.67
    banner.position.set(2.32,.99,-.69)
    cylinder(parts,.075,3.08,2.32,1.54,-.69,'iron')
    collider(frame,.18,3.05,.18,2.32,1.54,-.69)
    bake(parts,frame)
}
