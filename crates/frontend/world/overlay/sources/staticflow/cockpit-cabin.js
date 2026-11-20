// The wheel is about 34 cm from the eye. The dash brow, glass, bonnet and
// roof hoop sit at successively different depths around that same eye point.
import {
    BufferAttribute, BufferGeometry, Color, DataTexture, DoubleSide, Group, LinearFilter,
    LinearMipmapLinearFilter, MathUtils, Mesh, MeshBasicMaterial, RepeatWrapping, Vector3,
} from 'three/webgpu'

const referenceAspect = 1.6
const referenceLens = Math.tan(MathUtils.degToRad(68) / 2)
const light = new Vector3(-.30, .70, .64).normalize()

// Lay out the apertures in a reference camera gate, then build them at their
// actual depths. Side windows and the windscreen stay open to the world.
function point(u, v, depth) {
    return new Vector3((2 * u - 1) * referenceAspect * referenceLens * depth,
        (1 - 2 * v) * referenceLens * depth, -depth)
}

function surfaces(upholstery = false) {
    const positions = []
    const colors = []
    const uvs = []
    const base = new Color()
    const edge = new Vector3()
    const normal = new Vector3()
    const fittings = []
    function face(a, b, c, tone) {
        base.set(tone)
        normal.crossVectors(edge.subVectors(b, a), normal.subVectors(c, a)).normalize()
        const shade = .78 + .24 * Math.abs(normal.dot(light))
        for (const p of [a, b, c]) {
            positions.push(p.x, p.y, p.z)
            const throughGlass = MathUtils.clamp((-p.z - .24) / .44, 0, 1)
            const exposure = shade * (.84 + .16 * throughGlass)
            colors.push(base.r * exposure, base.g * exposure, base.b * exposure)
            if (upholstery) uvs.push(p.x * 11.6, (p.y * .68 - p.z) * 11.6)
        }
    }
    function quad(a, b, c, d, tone) {
        face(a, b, c, tone)
        face(a, c, d, tone)
    }
    function ribbon(a, b, tone) {
        for (let i = 1; i < a.length; i++) quad(a[i - 1], a[i], b[i], b[i - 1], tone)
    }
    // Each loop describes a closed silhouette at a new physical depth.
    function skin(rings, tones, cap) {
        for (let j = 1; j < rings.length; j++) {
            const a = rings[j - 1], b = rings[j]
            for (let i = 0; i < a.length; i++) {
                const next = (i + 1) % a.length
                quad(a[i], a[next], b[next], b[i], tones[j - 1])
            }
        }
        if (cap) {
            const last = rings.at(-1)
            for (let i = 2; i < last.length; i++) face(last[0], last[i - 1], last[i], cap)
        }
    }
    function tube(a, b, radius, tone, sides = 10, endRadius = radius) {
        const axis = new Vector3().subVectors(b, a).normalize()
        const tangent = new Vector3(0, 1, 0)
        if (Math.abs(axis.y) > .9) tangent.set(1, 0, 0)
        tangent.cross(axis).normalize()
        const bitangent = new Vector3().crossVectors(axis, tangent)
        const ring = (origin, r) => Array.from({ length: sides }, (_, i) => {
            const angle = i * 2 * Math.PI / sides
            return origin.clone().addScaledVector(tangent, r * Math.cos(angle))
                .addScaledVector(bitangent, r * Math.sin(angle))
        })
        skin([ring(a, radius), ring(b, endRadius)], [tone], tone)
    }
    function mesh(name, material) {
        const geometry = new BufferGeometry()
        geometry.setAttribute('position', new BufferAttribute(new Float32Array(positions), 3))
        geometry.setAttribute('color', new BufferAttribute(new Float32Array(colors), 3))
        if (upholstery) geometry.setAttribute('uv', new BufferAttribute(new Float32Array(uvs), 2))
        const mesh = new Mesh(geometry, material)
        mesh.name = name
        return mesh
    }
    // Keep moulded fittings proportionate when the A-posts are brought
    // inward on a phone. Update only these vertices when the viewport changes.
    function fitting(anchorV, draw) {
        const first = positions.length / 3
        draw()
        fittings.push({ first, end: positions.length / 3, anchorV })
    }
    function fitPanels(geometry, aspect) {
        const factor = MathUtils.clamp(aspect / referenceAspect, .48, 1)
        const attr = geometry.getAttribute('position')
        for (const { first, end, anchorV } of fittings) {
            for (let i = first; i < end; i++) {
                const anchorY = (1 - 2 * anchorV) * referenceLens * -positions[i * 3 + 2]
                attr.setY(i, anchorY + factor * (positions[i * 3 + 1] - anchorY))
            }
        }
        attr.needsUpdate = true
        geometry.computeBoundingSphere()
    }
    return { face, quad, ribbon, skin, tube, mesh, fitting, fitPanels }
}

const polygon = vertices => vertices.map(([u, v, z]) => point(u, v, z))

// Build a bevelled, chamfer-corner panel, including its return edge. A tilted
// or raked panel gives each corner a different depth through the projector.
function panel(surface, project, left, top, right, bottom, chamfer, depth, edge, face) {
    const outline = [[left + chamfer, top], [right - chamfer, top],
        [right, top + chamfer], [right, bottom - chamfer],
        [right - chamfer, bottom], [left + chamfer, bottom],
        [left, bottom - chamfer], [left, top + chamfer]]
    const back = outline.map(([x, y]) => project(x, y, 0))
    const front = outline.map(([x, y]) => project(x, y, depth))
    surface.skin([back, front], [edge], face)
}

function cabinFabric(shellMaterial) {
    // An uncoloured, seamless yarn tile. Mips average the fibres back into
    // the base cloth at a distance; the metal, switches and glass have no UVs.
    const size = 64
    const pixels = new Uint8Array(size * size * 4)
    for (let y = 0; y < size; y++) {
        for (let x = 0; x < size; x++) {
            const warp = ((x >> 2) + (y >> 2)) % 2 ? x : y
            const fibre = [243, 254, 249, 230][warp % 4]
            const value = fibre - ((x * 37 + y * 19 + x * y * 3) % 7)
            const i = 4 * (x + y * size)
            pixels.set([value, value, value, 255], i)
        }
    }
    const weave = new DataTexture(pixels, size, size)
    weave.wrapS = weave.wrapT = RepeatWrapping
    weave.generateMipmaps = true
    weave.magFilter = LinearFilter
    weave.minFilter = LinearMipmapLinearFilter
    weave.needsUpdate = true
    const material = new MeshBasicMaterial({ map: weave, vertexColors: true, side: DoubleSide, fog: false })
    material.color = shellMaterial.color
    return material
}

function roof(surface, fabric) {
    const stations = [-.20, .015, .14, .28, .47, .68, .84, .98, 1.20]
    const lip = [.22, .157, .116, .092, .087, .092, .114, .147, .214]
    const rear = stations.map(u => point(u, -.36, .23))
    const seam = stations.map((u, i) => point(u, lip[i] - .070, .385))
    const edge = stations.map((u, i) => point(u, lip[i] - .018, .468))
    const edgeFace = stations.map((u, i) => point(u, lip[i], .481))
    const gasket = stations.map((u, i) => point(u, lip[i] + .008, .489))
    fabric.ribbon(rear, seam, '#3e4140')
    fabric.ribbon(seam, edge, '#53564e')
    surface.ribbon(edge, edgeFace, '#333838')
    surface.ribbon(edgeFace, gasket, '#1f2728')

    // Padded sun visors, pivot pins, retaining clips and the overhead lamp.
    for (const [l, r, tilt] of [[.204, .404, -.021], [.706, .889, .022]]) {
        const visor = (u, v, offset) => point(u, v + tilt * (u - l) / (r - l), .436 + .07 * (u - l) + offset)
        fabric.fitting(.084, () => {
            panel(fabric, visor, l, .009, r, .114, .014, -.019, '#252b2c', '#686960')
            panel(fabric, (u, v, z) => visor(u, v, z - .020),
                l + .012, .020, r - .012, .098, .010, -.001, '#606159', '#63645c')
        })
        surface.fitting(.084, () => {
            surface.tube(visor(l + .01, .016, -.024), visor(l + .048, .016, -.024), .003, '#373b36')
            surface.tube(visor(r - .032, .016, -.023), visor(r - .014, .016, -.023), .0025, '#a6a398')
            // The elastic document strap crosses the passenger's visor.
            if (l > .5) {
                surface.quad(visor(r-.060,.020,-.024), visor(r-.050,.020,-.024),
                    visor(r-.046,.100,-.024), visor(r-.056,.100,-.024), '#4a504b')
            }
        })
    }
    const dome = (u, v, dz) => point(u, v, .40 + dz)
    panel(surface, dome, .442, -.018, .541, .042, .009, -.015, '#2b3232', '#464b48')
    panel(surface, (u, v, z) => dome(u, v, z - .016), .457, -.010, .526, .023, .005, -.002, '#292d2b', '#a79970')
    for (const u of [.466, .477, .488, .499, .510]) {
        surface.tube(dome(u, -.007, -.020), dome(u, .021, -.020), .00075, '#726d58', 5)
    }
}

function flank(surface, side) {
    // The driver is in the left seat. The near quarter-light is wider and
    // the passenger's door extends beyond the opposite edge of the camera.
    const project = (u, v, d) => side < 0
        ? point(.073 + .94 * u, v, .87 * d)
        : point(1.041 - u, v, 1.07 * d)
    const station = [[.159, .081, .468, .015], [.106, .337, .550, .018],
        [.053, .580, .632, .023], [.017, .722, .656, .031]]
    const band = (offset, z) => station.map(([u, v, d, w]) => project(u + offset * w, v, d + z))
    // Deep return on the glass side, moulded A-post trim, painted stamped
    // flange on the outside. The small secondary opening is the side glass.
    surface.ribbon(band(-1.20, .007), band(-.83, .003), '#452a29')
    surface.ribbon(band(-.83, .003), band(-.69, -.003), '#be6650')
    surface.ribbon(band(-.69, -.003), band(-.38, -.011), '#7b8077')
    surface.ribbon(band(-.38, -.011), band(.59, -.015), '#545953')
    surface.ribbon(band(.59, -.015), band(.83, -.004), '#343c3b')
    surface.ribbon(band(.83, -.004), band(1.13, .015), '#181f21')
    surface.ribbon(band(1.13, .015), band(1.23, .017), '#7b817c')

    // The steel hoop sits on the door side of the trim and turns up into
    // the roof. Metal collars and a bolted foot plate carry the hardpoints.
    const rail = station.map(([u, v, d, w]) => project(u - 1.80 * w, v, d - .030))
    for (let i = 1; i < rail.length; i++)
        surface.tube(rail[i - 1], rail[i], .0125, '#974c3d', 12)
    surface.tube(rail[0], project(.121, .062, .456), .015, '#503d35', 12)
    for (const [segment, t] of [[0,.62], [2,.55]]) {
        const a = rail[segment].clone().lerp(rail[segment + 1], t)
        const b = rail[segment].clone().lerp(rail[segment + 1], t + .11)
        surface.tube(a, b, .014, '#a5a59a', 12)
    }
    // The narrow upright at the front of the near door separates the
    // quarter light from the main side pane.
    surface.tube(project(-.085, .085, .368), project(.004, .694, .383), .0036, '#303a3b', 8)

    const sill = polygonForSide([[-.14,.627,.25],[-.045,.635,.30],[.015,.654,.396],[.088,.687,.585]])
    const top = polygonForSide([[-.14,.664,.241],[-.045,.670,.290],[.019,.689,.384],[.107,.713,.570]])
    const roll = polygonForSide([[-.14,.685,.239],[-.045,.687,.288],[.024,.707,.382],[.110,.729,.565]])
    const outer = polygonForSide([[-.14,.703,.242],[-.045,.708,.291],[.025,.727,.386],[.111,.746,.567]])
    const lower = polygonForSide([[-.14,1.32,.233],[-.035,1.28,.266],[.064,1.17,.365],[.153,1.01,.536]])
    surface.ribbon(sill, top, '#262d2d')
    surface.ribbon(top, roll, '#ad5c44')
    surface.ribbon(roll, outer, '#572f29')
    surface.ribbon(outer, lower, '#414745')
    function polygonForSide(vertices) { return vertices.map(([u,v,d]) => project(u,v,d)) }

    const insertTop = polygonForSide([[-.045,.747,.280],[.022,.755,.333],[.098,.776,.497]])
    const insertBottom = polygonForSide([[-.045,.907,.264],[.023,.922,.318],[.109,.876,.493]])
    surface.ribbon(insertTop, insertBottom, '#303839')
    // Door pull bridges its recessed pocket; a narrower silver latch sits
    // above it and a red woven release loop hangs from the mounting screw.
    // Inverse depth interpolates linearly across this angled planar door
    // card, so the release, pull and strap really sit on the same trim.
    const pocket = (u, v, z) => project(u, v + .22 * u, 1 / (3.45 - 8.4 * u) + z)
    panel(surface, pocket, -.026, .738, .108, .895, .016, -.004, '#21282a', '#3d4441')
    panel(surface, pocket, .010, .752, .081, .814, .008, -.009, '#5e635c', '#161e20')
    surface.tube(pocket(.024,.787,-.013), pocket(.067,.771,-.012), .0031, '#a7aaa0', 8)
    const grip = [[.013,.853,-.007],[.025,.874,-.031],[.074,.866,-.035],[.092,.842,-.008]]
        .map(([u,v,z])=>pocket(u,v,z))
    for (let i = 1; i < grip.length; i++) surface.tube(grip[i-1],grip[i],.0065,'#727770',8)
    const strap = [[.083,.793,-.011],[.081,.821,-.026],[.088,.832,-.024],[.091,.794,-.010]]
        .map(([u,v,z])=>pocket(u,v,z))
    for (let i = 1; i < strap.length; i++) surface.tube(strap[i-1],strap[i],.0028,'#ce684a',6)
    const speaker = (u,v,z) => project(u,v+.23*u,.284+.88*u+z)
    panel(surface,speaker,-.024,.920,.057,1.055,.011,-.009,'#62635d','#282f31')
    for (let i = 0; i < 6; i++) {
        const y = .941 + i * .015
        surface.tube(speaker(-.012,y,-.012),speaker(.043,y-.004,-.012),.0014,'#4b5150',6)
    }
}

function fascia(surface, fabric) {
    // Offset the passenger-side shelf and pull the corners back toward the
    // doors. The dash is sectioned from the windscreen seal down to the feet.
    const stations = [
        [-.08,.628,.510,.756,.366], [.076,.655,.562,.779,.388],
        [.183,.675,.611,.811,.412], [.310,.692,.658,.835,.435],
        [.436,.699,.687,.847,.449], [.534,.698,.691,.848,.451],
        [.621,.690,.679,.821,.441], [.722,.675,.641,.790,.425],
        [.843,.659,.593,.780,.407], [1.07,.626,.512,.738,.371],
    ]
    const row = (v, d) => stations.map(([x, glassY, glassD, edgeY, edgeD]) =>
        point(x, v(glassY, edgeY, x), d(glassD, edgeD, x)))
    const sealBack = row(y => y - .005, d => d + .010)
    const sealFront = row(y => y + .010, d => d - .006)
    const shelf = row((y, e) => y + (e-y)*.72, (d,e) => d + (e-d)*.80)
    const crown = row((y,e) => e-.010, (d,e) => e)
    const brow = row((y,e) => e+.018, (d,e) => e-.003)
    const lip = row((y,e) => e+.029, (d,e) => e+.005)
    const rail = row((y,e) => e+.11, (d,e) => e+.002)
    const foot = row((y,e) => e+.50, (d,e) => e+.080)
    surface.ribbon(sealBack, sealFront, '#283333')
    fabric.ribbon(sealFront, shelf, '#686b63')
    fabric.ribbon(shelf, crown, '#5d6059')
    surface.ribbon(crown, brow, '#383d3c')
    surface.ribbon(brow, lip, '#959083')
    surface.ribbon(lip, rail, '#404545')
    surface.ribbon(rail, foot, '#2d3333')
    // A compressed piping seam follows the dash brow all the way into the doors.
    const stitched = crown.map((vertex,i) => vertex.clone().lerp(shelf[i],.18))
    for (let i=1; i<stitched.length; i++) {
        const a=stitched[i-1], b=stitched[i]
        surface.tube(a,b,.0010,'#4a4e48',7)
        const stitches = Math.ceil(a.distanceTo(b) / .010)
        for (let j=0; j<stitches; j++) {
            surface.tube(a.clone().lerp(b,(j+.12)/stitches),a.clone().lerp(b,(j+.57)/stitches),
                .0011,'#686c60',6)
        }
    }

    // The matte central bonnet and stamped ridges are beyond the lower seal.
    // The outside is still visible between the blades and the windscreen.
    const hoodRear = polygon([[.092,.683,.607],[.205,.692,.662],[.356,.705,.744],
        [.538,.714,.772],[.750,.687,.678],[.916,.668,.586]])
    const hoodFront = polygon([[.141,.644,.917],[.245,.628,1.00],[.352,.621,1.036],
        [.545,.621,1.043],[.733,.622,1.00],[.876,.628,.868]])
    surface.ribbon(hoodFront,hoodRear,'#673f38')
    for (const [l, r] of [[.198,.242],[.731,.778]]) {
        surface.quad(point(l-.007,.657,.756),point(l+.008,.638,.966),
            point(r,.628,1.002),point(r+.016,.663,.771),'#895345')
        surface.face(point(r,.628,1.002),point(r+.024,.661,.758),point(r+.016,.663,.771),'#493a34')
    }
    surface.quad(point(.329,.651,.894),point(.346,.643,.936),
        point(.616,.643,.936),point(.640,.658,.888),'#302f2c')
    for (const u of [.371,.407,.443,.479,.515,.551,.587]) {
        surface.tube(point(u,.647,.919),point(u-.002,.653,.897),.0018,'#65635a',6)
    }

    // Padded glovebox recess and latch on the passenger side, separate from
    // the driver instruments and the taller center controls.
    const passenger = (u,v,d) => point(.781+(u-.690)*.82, v-.15*(u-.66), .405-.12*(u-.66)+d)
    surface.fitting(.855, () => {
        panel(surface, passenger, .690, .820, .888, .984, .014, -.012, '#73756e', '#242b2c')
        panel(surface, (u,v,d)=>passenger(u,v,d-.014), .695,.830,.882,.971,.012,-.003,'#333a38','#4e5350')
        panel(surface,(u,v,d)=>passenger(u,v,d-.020),.762,.851,.819,.871,.004,-.002,'#70736c','#191f21')
        surface.tube(passenger(.772,.860,-.024),passenger(.809,.856,-.025),.0024,'#a4a397',8)
        for (const u of [.716,.857])
            surface.tube(passenger(u,.947,-.017),passenger(u,.947,-.021),.0021,'#878b83',8)
    })

    // Fit the defroster returns and the passenger vent to the sloping shelf.
    // Its depth varies across the car and across each individual vent blade.
    const dashTop = (u, v, offset) => {
        const i = stations.findIndex(station => station[0] >= u)
        const a = stations[i - 1], b = stations[i]
        const t = (u - a[0]) / (b[0] - a[0])
        const [, glassY, glassD, edgeY, edgeD] = a.map((val,n) => MathUtils.lerp(val,b[n],t))
        const f = (v - glassY - .010) / (.72 * (edgeY - glassY) - .010)
        return point(u,v,MathUtils.lerp(glassD - .006, MathUtils.lerp(glassD,edgeD,.80), f) + offset - .004)
    }
    const vent = (u,v,d) => dashTop(u,v-.12*(u-.82),d)
    panel(surface, vent,.824,.704,.903,.735,.004,-.005,'#787d73','#272c2b')
    for (let i=0; i<6; i++) {
        const u=.830+i*.013
        surface.tube(vent(u,.708,-.007),vent(u+.003,.731,-.007),.0016,'#85887c',6)
    }
    for (const [u,v] of [[.232,.706],[.624,.708]]) {
        panel(surface,dashTop,u,v,u+.069,v+.011,.003,-.003,'#343a3a','#22292a')
        surface.tube(dashTop(u+.008,v+.005,-.004),dashTop(u+.061,v+.005,-.004),.0012,'#71756a',6)
    }
}

function centerConsole(surface) {
    // The face pitches up out of the transmission tunnel and yaws toward
    // the left seat. Its base carries a concertina gaiter and ball shifter.
    const face = (x,y,d) => point(.622 + .148*x + .041*y,
        .742 + .246*y - .021*x, .442 - .128*y + .033*x + d)
    panel(surface,face,0,0,1,1,.06,-.014,'#333938','#5d625c')
    const recess = (x,y,d) => face(x,y,d-.016)
    panel(surface,recess,.030,.042,.970,.517,.065,-.006,'#8c9086','#343c3b')
    for (const start of [.070,.520]) {
        panel(surface,(x,y,d)=>recess(x,y,d-.007),start,.092,start+.406,.439,.034,-.002,'#5b605a','#1b2325')
        for (const level of [.159,.244,.329]) {
            const flap = (x,y,d) => recess(x,y,d-.010)
            surface.quad(flap(start+.032,level,0),flap(start+.368,level,0),
                flap(start+.368,level+.029,-.001),flap(start+.032,level+.029,-.001),'#727970')
        }
        surface.tube(recess(start+.164,.244,-.014),recess(start+.242,.244,-.014),.0018,'#aaa897',7)
    }
    const tray = (x,y,d) => recess(x,y,d-.004)
    panel(surface,tray,.090,.570,.920,.958,.040,-.006,'#454c49','#272e2f')
    const center = tray(.59,.82,-.012)
    const knob = point(.755,.825,.307)
    const axis = knob.clone().sub(center)
    const ring = (t, width, depth) => {
        const origin = center.clone().addScaledVector(axis,t)
        return [[-1,-.7],[-.7,-1],[.7,-1],[1,-.7],[1,.7],[.7,1],[-.7,1],[-1,.7]]
            .map(([x,z])=>origin.clone().add(new Vector3(x*width,0,z*depth)))
    }
    const folds = [[0,.028,.016],[.08,.027,.016],[.15,.023,.013],[.20,.024,.014],
        [.29,.017,.010],[.33,.018,.011],[.42,.009,.007],[.45,.009,.006]]
    surface.skin(folds.map(parts=>ring(...parts)),['#68706a','#2b3233','#525854','#2c3132',
        '#555a55','#313737','#555b59'],'#313737')
    surface.tube(center.clone().addScaledVector(axis,.42), knob, .0031,'#a9aa9d',10)
    const knobRings = [[-.007,.003],[-.005,.007],[0,.009],[.005,.0085],[.008,.005]]
        .map(([dy,r])=>Array.from({length:12},(_,i)=>{
            const phi=i*Math.PI/6
            return knob.clone().add(new Vector3(r*Math.cos(phi),dy,r*Math.sin(phi)))
        }))
    surface.skin(knobRings,['#454842','#524c43','#6a5e4f','#7f705a'],'#83735e')
}

function wiperArm(material, shell, pivot, span) {
    const parts = surfaces()
    const a = point(pivot, .684, .640)
    const width = span * 2 * referenceAspect * referenceLens * .640
    // All joints use one sloping windscreen plane. The spring arm connects
    // the spindle to the bridge at the middle of the full-length rubber blade.
    const onGlass = (t, y, standOff = 0) => new Vector3(
        a.x + t * width, a.y + y, a.z + y * .46 + standOff)
    const knuckle = onGlass(.26, .011, .008)
    const bridge = onGlass(.65, .016, .008)
    parts.tube(a, knuckle, .0034, '#494f4e', 8, .0027)
    parts.tube(knuckle, bridge, .0024, '#3c4140', 8, .0018)
    parts.tube(onGlass(.34, .011), onGlass(1, .020), .0027, '#1f2627', 8)
    for (const [end, y] of [[.39, .012], [.91, .019]]) {
        parts.tube(bridge, onGlass(end, y, .002), .0016, '#69716e', 6)
    }
    shell.tube(a, onGlass(0, .003, .004), .0052, '#545954', 8)
    const arm = parts.mesh('windscreen-wiper', material)
    arm.geometry.translate(-a.x, -a.y, -a.z)
    arm.position.copy(a)
    return arm
}

function cabinMirror(material, shell) {
    const surface = surfaces()
    const mount = (u,v,d) => point(u,v,.485+d)
    panel(shell,mount,.542,.085,.565,.109,.004,-.014,'#24292b','#444b4b')
    const pivot = point(.554,.103,.466)
    const frameCenter = point(.593,.155,.381)
    shell.tube(pivot,frameCenter,.006,'#515652',10)
    shell.tube(frameCenter,point(.595,.160,.360),.009,'#262b2b',10)
    const face = (u,v,d) => point(u, v+.047*(u-.593), .354+.062*(u-.593)+d)
    panel(surface,face,.515,.137,.677,.199,.008,-.009,'#191e20','#363d3d')
    panel(surface,(u,v,d)=>face(u,v,d-.010),.520,.143,.672,.192,.005,-.0008,'#92978d','#58676a')
    // A recessed rear-cabin view in the mirror glass: headlining, rear
    // window reveal, two seats and the X-brace between their shoulder belts.
    const rear = (u,v,d) => face(u,v,d-.0115)
    const roofEdge = [[.524,.147],[.663,.150],[.659,.157],[.530,.156]]
    const rearWindow = [[.537,.150],[.649,.152],[.657,.181],[.530,.179]]
    for (const [corners, tone] of [[roofEdge,'#394647'],[rearWindow,'#a2b2ae']])
        surface.quad(...corners.map(([u,v])=>rear(u,v,0)),tone)
    surface.quad(rear(.531,.177,-.001),rear(.656,.178,-.001),rear(.656,.181,-.001),rear(.531,.180,-.001),'#444f4f')
    surface.tube(rear(.540,.151,-.001),rear(.566,.181,-.001),.0009,'#786b5b',7)
    surface.tube(rear(.651,.153,-.0015),rear(.627,.181,-.0015),.0009,'#857867',7)
    for (const u of [.539,.645]) {
        panel(surface,(x,y,d)=>rear(x,y,d-.002),u-.012,.163,u+.012,.180,.004,-.0018,'#303d3e','#475455')
        panel(surface,(x,y,d)=>rear(x,y,d-.003),u-.017,.179,u+.017,.188,.003,-.0014,'#30393b','#394748')
    }
    const mirror = new Group()
    mirror.name = 'rear-cabin-mirror'
    mirror.position.copy(frameCenter)
    const frame = surface.mesh('mirror-housing-and-rear-seat-silhouettes', material)
    frame.geometry.translate(-frameCenter.x, -frameCenter.y, -frameCenter.z)
    mirror.add(frame)

    // Only the space beyond the rear glass receives the additional camera's
    // image. Render-target UVs start at the top in Three's node materials.
    // Reverse U so an overtaking car stays on the driver's physical side.
    const glass = new BufferGeometry()
    glass.setAttribute('position', new BufferAttribute(new Float32Array(rearWindow.flatMap(
        ([u, v]) => rear(u, v, -.0005).sub(frameCenter).toArray())), 3))
    glass.setAttribute('uv', new BufferAttribute(new Float32Array([1,0, 0,0, 0,1, 1,1]), 2))
    glass.setIndex([0, 1, 2, 0, 2, 3])
    const pane = new Mesh(glass, new MeshBasicMaterial({ side: DoubleSide, fog: false }))
    pane.name = 'rear-window-reflection'
    pane.visible = false
    mirror.add(pane)
    return { mirror, setRearViewTexture(texture) {
        if (pane.material.map !== texture) {
            pane.material.map = texture
            pane.material.needsUpdate = true
        }
        pane.visible = Boolean(texture)
    } }
}

export function createCockpitCabin() {
    const material = new MeshBasicMaterial({ vertexColors: true, side: DoubleSide, fog: false })
    const group = new Group()
    group.name = 'cockpit-cabin'
    const body = surfaces()
    const upholstery = surfaces(true)
    roof(body, upholstery)
    flank(body,-1)
    flank(body,1)
    fascia(body, upholstery)
    centerConsole(body)
    const wipers = [wiperArm(material, body, .205, .218), wiperArm(material, body, .587, .229)]
    const { mirror, setRearViewTexture } = cabinMirror(material, body)
    const structure = body.mesh('roof-pillars-doors-and-dash', material)
    const trim = upholstery.mesh('woven-headliner-and-padded-dash', cabinFabric(material))
    group.add(structure, trim, ...wipers, mirror)
    const spindle = new Vector3(0, -.46, 1).normalize()
    let lastAspect, lastFov, lastTime
    let phase = 0
    return { group, shellMaterial: material, setRearViewTexture, update(aspect, fov, telemetry) {
        if (aspect !== lastAspect || fov !== lastFov) {
            const lens = Math.tan(MathUtils.degToRad(fov) / 2) / referenceLens
            group.scale.set(lens * aspect / referenceAspect, lens, 1)
            if (aspect !== lastAspect) {
                body.fitPanels(structure.geometry, aspect)
                upholstery.fitPanels(trim.geometry, aspect)
                // Keep the housing wide and shallow at its roof stalk while
                // the side panes and A-posts move in on a narrow viewport.
                mirror.scale.x = MathUtils.clamp(referenceAspect / aspect, .60, 1.65)
                mirror.scale.y = MathUtils.clamp(aspect / 1.35, .67, 1)
            }
            lastAspect = aspect
            lastFov = fov
        }
        const dt = lastTime === undefined ? 0 : MathUtils.clamp(telemetry.elapsed - lastTime, 0, .15)
        lastTime = telemetry.elapsed
        if (telemetry.rain > .09 || phase > 0) {
            phase += dt * (.68 + .67 * telemetry.rain)
            if (phase >= 1) phase = telemetry.rain > .09 ? phase % 1 : 0
            const sweep = MathUtils.lerp(.55, .96, MathUtils.smoothstep(aspect, .54, 1.55))
            const angle = (1 - Math.cos(phase * Math.PI * 2)) * .5 * sweep
            for (const arm of wipers) arm.quaternion.setFromAxisAngle(spindle, angle)
        }
    } }
}
