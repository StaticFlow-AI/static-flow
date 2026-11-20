// The cluster is mounted to the dashboard in the steering column's parent space.
// All printed artwork and display glyphs share a single immutable texture.
import {
    BoxGeometry, BufferAttribute, BufferGeometry, CanvasTexture, CircleGeometry,
    CylinderGeometry, ExtrudeGeometry, Group, Mesh, MeshBasicMaterial, PlaneGeometry,
    RingGeometry, Shape, ShapeGeometry, SRGBColorSpace, TorusGeometry,
} from 'three/webgpu'
import { assemble, paintPart } from './cockpit-mesh.js'

const width = 1024, height = 512
const dial = { x: 2, y: 2, w: 460, h: 460 }
const display = { x: 482, y: 4, w: 516, h: 260 }
const displayWidth = .149, displayHeight = .075
const displayX = .085, displayY = .008
const dialX = -.065, dialY = .006, radius = .071
const maximum = 160
const sweep = Math.PI * 1.5
const segments = ['abcdef', 'bc', 'abdeg', 'abcdg', 'bcfg', 'acdfg', 'acdefg', 'abc', 'abcdefg', 'abcdfg']
const clamp = (value, low, high) => Math.min(high, Math.max(low, value))

function printDial(ctx) {
    ctx.save()
    ctx.translate(dial.x + dial.w / 2, dial.y + dial.h / 2)
    const lacquer = ctx.createRadialGradient(-65, -100, 5, 0, 0, 230)
    lacquer.addColorStop(0, '#303d3b')
    lacquer.addColorStop(.66, '#1c2729')
    lacquer.addColorStop(1, '#0b1115')
    ctx.fillStyle = lacquer
    ctx.beginPath()
    ctx.arc(0, 0, 229, 0, Math.PI * 2)
    ctx.fill()
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    for (let speed = 0; speed <= maximum; speed += 5) {
        const a = -Math.PI * 1.25 + sweep * speed / maximum
        const c = Math.cos(a), s = Math.sin(a)
        const major = speed % 20 === 0
        const inner = major ? 174 : speed % 10 === 0 ? 184 : 194
        ctx.strokeStyle = speed >= 140 ? '#efa475' : '#efead2'
        ctx.lineWidth = major ? 5 : 2.5
        ctx.beginPath()
        ctx.moveTo(c * inner, s * inner)
        ctx.lineTo(c * 204, s * 204)
        ctx.stroke()
        if (major) {
            ctx.font = '600 35px system-ui,sans-serif'
            ctx.fillStyle = '#fbf5dc'
            ctx.fillText(String(speed), c * 143, s * 143)
        }
    }
    ctx.strokeStyle = '#cf8663'
    ctx.lineWidth = 3
    ctx.beginPath()
    ctx.arc(0, 0, 214, -Math.PI * 1.25 + sweep * 140 / maximum, Math.PI / 4)
    ctx.stroke()
    ctx.fillStyle = '#d4ab75'
    ctx.font = 'italic 24px Georgia,serif'
    ctx.fillText('α / β', 0, -65)
    ctx.fillStyle = '#deebdf'
    ctx.font = '600 30px system-ui,sans-serif'
    ctx.fillText('km/h', 0, 97)
    ctx.beginPath()
    ctx.arc(0, 0, 224, 0, Math.PI * 2)
    ctx.clip()
    const reflection = ctx.createLinearGradient(-140,-180,50,144)
    reflection.addColorStop(0,'rgba(202,225,219,.13)')
    reflection.addColorStop(.46,'rgba(202,225,219,.035)')
    reflection.addColorStop(.47,'rgba(202,225,219,0)')
    ctx.fillStyle = reflection
    ctx.fillRect(-230,-230,460,460)
    ctx.restore()
}

const trackX = 101, trackEnd = 329
const throttleY = 182, brakeY = 224
function printDisplay(ctx) {
    ctx.save()
    ctx.translate(display.x, display.y)
    const phosphor = ctx.createLinearGradient(0, 0, 0, display.h)
    phosphor.addColorStop(0, '#121b1c')
    phosphor.addColorStop(.68, '#202722')
    phosphor.addColorStop(1, '#131a18')
    ctx.fillStyle = phosphor
    ctx.fillRect(0, 0, display.w, display.h)
    ctx.fillStyle = '#405049'
    ctx.fillRect(350, 16, 2, 126)
    ctx.fillRect(15, 148, 485, 2)
    ctx.fillStyle = '#c9d8c7'
    ctx.font = '600 32px system-ui,sans-serif'
    ctx.fillText('km/h', 267, 105)
    ctx.font = 'bold 26px monospace'
    for (const [name, y] of [['THR', throttleY], ['BRK', brakeY]]) {
        ctx.fillStyle = '#b5bfae'
        ctx.fillText(name, 23, y + 9)
        ctx.fillStyle = '#38443b'
        ctx.fillRect(trackX, y - 4, trackEnd - trackX, 8)
        for (let i = 0; i <= 4; i++) ctx.fillRect(trackX + (trackEnd - trackX) * i / 4, y + 8, 2, 5)
    }
    ctx.restore()
}

function digitRegion(n) { return { x: 485 + n * 47, y: 282, w: 42, h: 75 } }
function gearRegion(index) { return { x: 490 + index * 65, y: 383, w: 58, h: 84 } }
function createAtlas() {
    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    const ctx = canvas.getContext('2d')
    printDial(ctx)
    printDisplay(ctx)
    // Cut-glass digits. Each cell includes the dark, unlit segment electrodes.
    const bars = [[21,7,0],[36,21,1],[36,53,1],[21,68,0],[6,53,1],[6,21,1],[21,37,0]]
    for (let n = 0; n < 10; n++) {
        const cell = digitRegion(n)
        bars.forEach(([x, y, vertical], i) => {
            ctx.save()
            ctx.translate(cell.x + x, cell.y + y)
            if (vertical) ctx.rotate(Math.PI / 2)
            ctx.fillStyle = segments[n].includes('abcdefg'[i]) ? '#ffc585' : '#3b372a'
            ctx.beginPath()
            for (const [j, [px, py]] of [[-14,0],[-10,-3.7],[10,-3.7],[14,0],[10,3.7],[-10,3.7]].entries())
                j ? ctx.lineTo(px,py) : ctx.moveTo(px,py)
            ctx.closePath()
            ctx.fill()
            ctx.restore()
        })
    }
    for (const [i, letter] of ['D','N','R'].entries()) {
        const cell = gearRegion(i)
        ctx.fillStyle = letter === 'R' ? '#ffae78' : '#a2e4d2'
        ctx.textAlign = 'center'
        ctx.textBaseline = 'middle'
        ctx.font = 'bold 76px monospace'
        ctx.fillText(letter, cell.x + cell.w / 2, cell.y + cell.h / 2)
    }
    const texture = new CanvasTexture(canvas)
    texture.colorSpace = SRGBColorSpace
    return texture
}

function setUV(geometry, region) {
    const uv = geometry.attributes.uv
    const values = geometry.userData.baseUV ??= uv.array.slice()
    for (let i = 0; i < uv.count; i++)
        uv.setXY(i, (region.x + values[i * 2] * region.w) / width,
            1 - (region.y + (1 - values[i * 2 + 1]) * region.h) / height)
    uv.needsUpdate = true
}

function outline(points) {
    const shape = new Shape()
    shape.moveTo(...points[0])
    for (const point of points.slice(1)) shape.lineTo(...point)
    shape.closePath()
    return shape
}

function mount(parts) {
    const profile = [[-.176,-.095],[.179,-.095],[.179,.036],[.145,.072],[.016,.076],[-.012,.092],[-.122,.092],[-.169,.065],[-.178,.032]]
    const shell = new ExtrudeGeometry(outline(profile), { depth: .062, bevelEnabled: true, bevelSegments: 2, bevelThickness: .004, bevelSize: .004 })
    paintPart(parts, shell.translate(0, 0, -.063), '#343b3e')

    // The hood tapers back towards the screen. Its projecting lip and black
    // felt underside form a real recess above the gauge and the readout.
    const rail = [[-.172,.003],[-.166,.055],[-.128,.087],[-.112,.093],[-.018,.093],[.015,.076],[.145,.072],[.173,.046],[.178,.010]]
    for (let i = 1; i < rail.length; i++) {
        const [ax,ay] = rail[i-1], [bx,by] = rail[i]
        const aw = ax * .89, bw = bx * .89
        const surfaces = [
            [[aw,ay-.009,-.069],[bw,by-.009,-.069],[bx,by,.044],[ax,ay,.044], '#475053'],
            [[ax,ay,.044],[bx,by,.044],[bx,by-.004,.046],[ax,ay-.004,.046], '#656966'],
            [[ax,ay-.004,.046],[bx,by-.004,.046],[bw,by-.014,-.055],[aw,ay-.014,-.055], '#1a2123'],
        ]
        for (const [a,b,c,d,tone] of surfaces) {
            const g = new BufferGeometry()
            g.setAttribute('position', new BufferAttribute(new Float32Array([...a,...c,...b,...a,...d,...c]), 3))
            g.computeVertexNormals()
            paintPart(parts,g,tone)
        }
    }
    const cylinder = (x,y,z,r,depth,tone) => paintPart(parts, new CylinderGeometry(r,r,depth,64).rotateX(Math.PI/2).translate(x,y,z),tone)
    cylinder(dialX,dialY,.009,.080,.015,'#232b2c')
    paintPart(parts,new RingGeometry(radius,.079,64).translate(dialX,dialY,.020),'#949894')
    paintPart(parts,new TorusGeometry(.077,.0029,8,64).translate(dialX,dialY,.021),'#b9bcb4')
    paintPart(parts,new TorusGeometry(.0714,.0020,8,64).translate(dialX,dialY,.022),'#252c2a')
    // Raised display bezel, with notched captive screws at the two outer corners.
    paintPart(parts,new BoxGeometry(.161,.087,.017).translate(displayX,displayY,.009),'#5a6260')
    paintPart(parts,new BoxGeometry(.155,.080,.004).translate(displayX,displayY,.019),'#1b2220')
    // A small demister outlet under the display. The deep throat, angled
    // louvres and knurled thumbwheel catch light at different angles.
    const ventX = .085, ventY = -.059
    const box = (w,h,d,x,y,z,color,tilt=0) =>
        paintPart(parts,new BoxGeometry(w,h,d).rotateX(tilt).translate(x,y,z),color)
    box(.150,.031,.008,ventX,ventY,.005,'#555c5a')
    box(.145,.027,.003,ventX,ventY,.010,'#151d1f')
    for (const offset of [-.008,0,.008])
        box(.116,.0025,.007,ventX-.009,ventY+offset,.013,'#646d69',-.32)
    box(.0028,.025,.004,ventX+.052,ventY,.013,'#41494a')
    box(.011,.019,.005,ventX+.063,ventY,.013,'#6a6458')
    for (const offset of [-.005,0,.005])
        box(.009,.0014,.0015,ventX+.063,ventY+offset,.016,'#b0a389')
    for (const x of [-.16,.165]) {
        cylinder(x,-.079,.005,.0045,.004,'#a1a59f')
        paintPart(parts,new BoxGeometry(.005,.0011,.001).rotateZ(.36).translate(x,-.079,.0075),'#242927')
    }
}

function panelX(px) { return displayX + (px/display.w - .5) * displayWidth }
function panelY(py) { return displayY + (.5 - py/display.h) * displayHeight }

export function createCockpitInstruments() {
    const group = new Group()
    group.name = 'cockpit-instruments'
    group.position.set(0, -.267, -.898)
    group.rotation.x = -.14
    const metal = new MeshBasicMaterial({vertexColors:true, toneMapped:false, fog:false})
    const ink = new MeshBasicMaterial({map:createAtlas(), alphaTest:.08, toneMapped:false, fog:false})
    const parts = []
    mount(parts)
    group.add(assemble(parts, metal))

    function face(geometry, cell, x, y, z, name) {
        setUV(geometry, cell)
        const mesh = new Mesh(geometry, ink)
        mesh.name = name
        mesh.position.set(x,y,z)
        group.add(mesh)
        return mesh
    }
    face(new CircleGeometry(radius,64),dial,dialX,dialY,.018,'speed-dial')
    face(new PlaneGeometry(displayWidth,displayHeight),display,displayX,displayY,.022,'telemetry-panel')

    const pointer = []
    paintPart(pointer,new ShapeGeometry(outline([[-.003,-.014],[.003,-.014],[.0018,.046],[.0005,.063],[-.001,.063],[-.0028,.048]])),'#ff884e')
    paintPart(pointer,new ShapeGeometry(outline([[-.0015,.048],[.0017,.048],[.0005,.063],[-.001,.063]])).translate(0,0,.0004),'#ffe9b2')
    paintPart(pointer,new CylinderGeometry(.008,.009,.004,24).rotateX(Math.PI/2).translate(0,0,.0015),'#777f7c')
    paintPart(pointer,new CylinderGeometry(.0063,.0063,.002,24).rotateX(Math.PI/2).translate(0,0,.004),'#272d2b')
    const needle = assemble(pointer,metal)
    needle.name = 'speed-needle'
    needle.position.set(dialX,dialY,.0225)
    group.add(needle)
    const glass = new Mesh(new CircleGeometry(radius,64),new MeshBasicMaterial({
        color:'#a8dbd8', transparent:true, opacity:.047, depthWrite:false, toneMapped:false, fog:false,
    }))
    glass.name = 'speedometer-glass'
    glass.position.set(dialX,dialY,.030)
    group.add(glass)

    const digits = new Group()
    digits.name = 'speed-readout'
    for (const x of [51,130,209]) {
        const digit = face(new PlaneGeometry(.019,.033),digitRegion(0),panelX(x),panelY(83),.0228,'speed-digit')
        digits.add(digit)
    }
    group.add(digits)
    const gear = face(new PlaneGeometry(.021,.032),gearRegion(1),panelX(389),panelY(82),.0228,'gear-display')
    const indicators = [[throttleY,'#93d4b7','throttle-indicator'],[brakeY,'#f59c70','brake-indicator']].map(([y,color,name]) => {
        const length = (trackEnd - trackX) / display.w * displayWidth
        const fill = new Mesh(new PlaneGeometry(length,.0024).translate(length/2,0,0),
            new MeshBasicMaterial({color, toneMapped:false, fog:false}))
        fill.name = name
        fill.position.set(panelX(trackX),panelY(y),.0228)
        group.add(fill)
        return fill
    })

    function update({speedKmh,drive,throttle,brake}) {
        const speed = Math.abs(speedKmh)
        const angle = sweep/2 - sweep * clamp(speed,0,maximum) / maximum
        if (needle.rotation.z !== angle) needle.rotation.z = angle
        const reading = Math.round(Math.min(speed,999))
        if (digits.userData.speedKmh !== reading) {
            digits.userData.speedKmh = reading
            const value = String(reading).padStart(3,' ')
            digits.children.forEach((digit,i) => {
                digit.visible = value[i] !== ' '
                if (digit.visible && digit.userData.glyph !== value[i]) {
                    digit.userData.glyph = value[i]
                    setUV(digit.geometry,digitRegion(Number(value[i])))
                }
            })
        }
        if (drive !== gear.userData.drive) {
            gear.userData.drive = drive
            setUV(gear.geometry,gearRegion(drive === 'D' ? 0 : drive === 'R' ? 2 : 1))
        }
        indicators[0].scale.x = clamp(throttle,0,1)
        indicators[1].scale.x = clamp(brake,0,1)
    }
    update({speedKmh:0,drive:'N',throttle:0,brake:0})
    return {group,update}
}
