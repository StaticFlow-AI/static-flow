import * as THREE from 'three/webgpu'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'
import { Area } from '../Game/World/Areas/Area.js'
import { InteractivePoints } from '../Game/InteractivePoints.js'
import { WorldlineRoute } from './worldline-route.js'

const steel = new THREE.MeshStandardMaterial({ color: '#354846', roughness: .68 })
const ceramic = new THREE.MeshStandardMaterial({ color: '#e5d7ad', roughness: .77 })
const brass = new THREE.MeshStandardMaterial({ color: '#b6874d', metalness: .32, roughness: .4 })
const lamp = new THREE.MeshBasicMaterial({ color: '#ffb752' })
const colors = { alpha: '#80e4ce', beta: '#ffb058', gate: '#fff0bb' }

function screen(width, height) {
    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    const texture = new THREE.CanvasTexture(canvas)
    texture.colorSpace = THREE.SRGBColorSpace
    return { context: canvas.getContext('2d'), texture }
}

function drawMeter(context, text) {
    const { width, height } = context.canvas
    context.fillStyle = '#211711'
    context.fillRect(0, 0, width, height)
    context.font = '500 135px "Courier New", monospace'
    context.textAlign = 'center'
    context.textBaseline = 'middle'
    for (const [i, char] of [...text].entries()) {
        const x = (i + .5) * width / 8
        context.strokeStyle = '#503427'
        context.lineWidth = 1.8
        context.strokeText('8', x, height / 2 + 8)
        context.fillStyle = '#ffb464'
        context.shadowColor = '#ff681b'
        context.shadowBlur = 15
        context.fillText(char, x, height / 2 + 8)
        context.shadowBlur = 0
        context.fillStyle = '#f4cba0'
        context.fillRect(x - 26, 10, 25, 2)
    }
}

function buildInstrument(origin, objects) {
    const group = new THREE.Group()
    group.name = 'WorldlineObservatory'
    const batches = new Map()
    function part(geometry, material, x, y, z, angle = 0) {
        geometry.rotateX(angle).translate(x, y, z)
        if (!batches.has(material)) batches.set(material, [])
        batches.get(material).push(geometry)
    }
    function box(material, x, y, z, w, h, d) {
        part(new THREE.BoxGeometry(w, h, d), material, x, y, z)
    }
    function cylinder(material, x, y, z, top, base, height, segments = 32) {
        part(new THREE.CylinderGeometry(top, base, height, segments), material, x, y, z)
    }

    // Flush approach to a calibration turntable, with instrument rack behind it.
    cylinder(steel, 0, .03, 0, 3.65, 3.72, .12, 64)
    for (const radius of [2.78, 3.46]) {
        part(new THREE.TorusGeometry(radius, .035, 5, 72), brass, 0, .11, 0, Math.PI / 2)
    }
    for (let n = 0; n < 48; n++) {
        const a = n * Math.PI / 24
        const tick = new THREE.BoxGeometry(.044, .022, n % 4 === 0 ? .32 : .14)
        tick.rotateY(a)
        part(tick, n % 4 === 0 ? lamp : ceramic, Math.sin(a) * 3.22, .12, Math.cos(a) * 3.22)
    }
    const panelZ = -3.75
    box(steel, 0, .76, panelZ, 7.4, 1.5, 1.34)
    box(ceramic, 0, 1.55, panelZ, 7.65, .23, 1.48)
    box(brass, 0, .23, panelZ, 7.65, .17, 1.46)
    for (let i = 0; i < 8; i++) {
        const x = (i - 3.5) * .87
        cylinder(brass, x, 1.76, panelZ, .39, .4, .22)
        cylinder(steel, x, 2.46, panelZ, .394, .394, 1.28)
        cylinder(brass, x, 3.12, panelZ, .28, .39, .12)
        // Side rails and a crown frame each glowing cathode.
        for (const side of [-1, 1]) cylinder(brass, x + side * .365, 2.47, panelZ + .21, .022, .022, 1.27, 6)
    }
    for (const x of [-3.1, -2.7, -2.3, -1.9, -1.5]) {
        box(brass, x, .82, panelZ + .69, .09, .65, .05)
    }
    for (const x of [2.25, 2.64, 3.03]) {
        part(new THREE.CylinderGeometry(.13, .13, .1, 12), brass, x, .84, panelZ + .74, Math.PI / 2)
    }

    for (const [material, parts] of batches) {
        const geometry = mergeGeometries(parts)
        parts.forEach(part => part.dispose())
        group.add(new THREE.Mesh(geometry, material))
    }
    const display = screen(1024, 164)
    const displayFace = new THREE.Mesh(new THREE.PlaneGeometry(6.96, 1.14),
        new THREE.MeshBasicMaterial({ map: display.texture }))
    displayFace.position.set(0, 2.46, panelZ + .398)
    group.add(displayFace)
    // Gaps between the eight tube windows expose the metal enclosure.
    for (let i = 1; i < 8; i++) {
        const rib = new THREE.Mesh(new THREE.BoxGeometry(.105, 1.22, .09), brass)
        rib.position.set((i - 4) * .87, 2.46, panelZ + .43)
        group.add(rib)
    }
    const badge = screen(640, 108)
    badge.context.fillStyle = '#e7dabe'
    badge.context.fillRect(0, 0, 640, 108)
    badge.context.fillStyle = '#293e3a'
    badge.context.textAlign = 'center'
    badge.context.font = 'bold 48px sans-serif'
    badge.context.fillText('世界线观测站', 320, 55)
    badge.context.font = '23px monospace'
    badge.context.fillText('STATICFLOW  /  DIVERGENCE METER', 320, 91)
    const plaque = new THREE.Mesh(new THREE.PlaneGeometry(2.95, .54), new THREE.MeshBasicMaterial({ map: badge.texture }))
    plaque.position.set(.2, .84, panelZ + .68)
    group.add(plaque)
    // A broad arrow at the rim remains visible when a car is parked on the dial.
    const arrowhead = new THREE.BufferGeometry()
    arrowhead.setAttribute('position', new THREE.Float32BufferAttribute([
        0, .16, 3.42, -.37, .16, 2.73, .37, .16, 2.73,
    ], 3))
    const pointer = new THREE.Mesh(arrowhead, new THREE.MeshBasicMaterial({ color: '#7df4e4', side: THREE.DoubleSide }))
    group.add(pointer)
    // The two attraction fields share an origin, diverge and meet again on
    // the dial. Tuning a branch energizes its trace. Reaching the gate lights both.
    const traces = {}
    for (const [kind, side] of [['alpha', -1], ['beta', 1]]) {
        const points = Array.from({ length: 21 }, (_, i) => {
            const t = i / 20
            return new THREE.Vector3((t - .5) * 5.2, .145, Math.sin(Math.PI * t) * side * .98)
        })
        const material = new THREE.MeshBasicMaterial({ color: '#655b4d' })
        group.add(new THREE.Mesh(new THREE.TubeGeometry(new THREE.CatmullRomCurve3(points), 28, .045, 5, false), material))
        traces[kind] = material
    }
    const colliders = [
        { shape: 'cylinder', parameters: [.06, 3.65], position: { x: 0, y: .03, z: 0 } },
        { shape: 'cuboid', parameters: [3.82, 1.58, .72], position: { x: 0, y: 1.58, z: panelZ } },
    ]
    objects.add({ model: group, updateMaterials: false }, { type: 'fixed', position: origin, colliders })
    return { group, display, pointer, traces }
}

export class WorldlineObservatory extends Area {
    constructor(model) {
        // Keep Folio's real map bounds, respawn location and terrain. Remove the
        // altar, its hidden colliders and skull before Area registers any bodies.
        for (const node of [...model.children]) {
            if (!node.name.startsWith('refZone') && node.name !== 'refAltar') model.remove(node)
        }
        super(model)
        this.origin = this.references.items.get('altar')[0].position.clone()
        // Arrive on the calibration pad with the console in reach of the car.
        this.game.respawns.getByName('altar').position.copy(this.origin).add(new THREE.Vector3(0, 4, 1.3))
        const instrument = buildInstrument(this.origin, this.game.objects)
        this.display = instrument.display
        this.pointer = instrument.pointer
        this.traces = instrument.traces
        this.objects.hideable.push(instrument.group)
        this.count = 0
        this.pulse = new THREE.Mesh(new THREE.TorusGeometry(1, .025, 6, 64),
            new THREE.MeshBasicMaterial({ color: '#83e8df', transparent: true, depthWrite: false, opacity: 0 }))
        this.pulse.rotation.x = Math.PI / 2
        this.pulse.position.set(0, .15, 0)
        instrument.group.add(this.pulse)
        this.chime = this.game.audio.register({ path: 'sounds/bell/Death Hit.mp3', volume: .32, positions: [this.origin], distanceFade: 18 })
        this.route = new WorldlineRoute(this.game, () => {
            this.startedAt = this.game.ticker.elapsed
            this.refresh()
        }, () => this.shift())
        const buttons = this.game.inputs.interactiveButtons
        this.point = this.game.interactivePoints.create(this.origin.clone().add(new THREE.Vector3(0, 1.3, -.7)),
            '调谐世界线', InteractivePoints.ALIGN_RIGHT, InteractivePoints.STATE_CONCEALED,
            () => this.route.openStation(), () => buttons.addItems(['interact']),
            () => buttons.removeItems(['interact']), () => buttons.removeItems(['interact']))
        this.events.on('boundingIn', () => this.game.achievements.setProgress('areas', 'altar'))
        const sync = data => {
            if ((data.type === 'init' || data.type === 'cataclysmUpdate') && data.cataclysmCount !== this.count)
                this.changeTo(data.cataclysmCount)
        }
        this.game.server.events.on('message', sync)
        if (this.game.server.initData) sync(this.game.server.initData)
        this.game.map.modal.events.on('open', () => this.markSignal())
        this.refresh()
    }

    refresh() {
        const [reading, name] = this.route.reading(this.count)
        const readout = document.querySelector('[data-observatory-reading]')
        readout.textContent = reading + ' %  ·  ' + name
        drawMeter(this.display.context, reading)
        this.display.texture.needsUpdate = true
        const branch = this.route.progress.active
        const signalColor = colors[branch] || '#80e4ce'
        for (const [kind, trace] of Object.entries(this.traces)) {
            trace.color.set(branch === kind || branch === 'gate' ? colors[kind] : '#655b4d')
        }
        this.pulse.material.color.set(signalColor)
        this.pointer.material.color.set(signalColor)
        const destination = this.route.stations[this.route.target]?.base
        this.pointer.visible = Boolean(destination)
        if (destination) this.pointer.rotation.y = Math.atan2(destination.x - this.origin.x, destination.z - this.origin.z)
        this.game.map.element.style.setProperty('--signal-color', colors[this.route.target] || signalColor)
        this.markSignal()
        this.route.refresh(this.count)
    }

    markSignal() {
        for (const button of this.game.map.element.querySelectorAll('[data-station]')) {
            const active = button.dataset.station === this.route.signal
            button.classList.toggle('is-worldline-signal', active)
            if (active) button.setAttribute('aria-description', '当前世界线的信号源')
            else button.removeAttribute('aria-description')
        }
    }

    changeTo(count) {
        this.count = count
        this.startedAt = this.game.ticker.elapsed
        this.refresh()
    }

    async shift() {
        if (this.pending || this.game.ticker.elapsed - (this.lastShift ?? -10) < 3) return
        this.pending = true
        try {
            if (this.game.server.connected) {
                if (!await this.game.server.send({ type: 'cataclysmInsert' })) return
            } else this.changeTo(this.count + 1)
            this.lastShift = this.game.ticker.elapsed
            this.chime.play()
            this.game.achievements.setProgress('sacrifice', 1)
        } finally { this.pending = false }
    }

    update() {
        if (this.startedAt === undefined) return
        const t = Math.min(1, (this.game.ticker.elapsed - this.startedAt) / 1.6)
        this.pulse.material.opacity = (1 - t) * .95
        this.pulse.scale.setScalar(.3 + 3.2 * t)
        this.pulse.position.y = .17 + .4 * Math.sin(Math.PI * t)
        if (t === 1) this.startedAt = undefined
    }
}
