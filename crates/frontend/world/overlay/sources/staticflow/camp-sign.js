// Bevelled letter sculptures at the trailhead. Each letter uses the original
// scene object's rigid body, palette, audio trigger and reset position.
import { ExtrudeGeometry, Path, Shape, Vector2, Vector3 } from 'three/webgpu'

const outlines = {
    S: ['0,0 .82,0 1,.17 1,.48 .82,.62 .26,.62 .26,.79 1,.79 1,1 .18,1 0,.83 0,.51 .17,.39 .74,.39 .74,.21 0,.21'],
    T: ['.36,0 .64,0 .64,.77 1,.77 1,1 0,1 0,.77 .36,.77'],
    A: ['0,0 .26,0 .33,.22 .69,.22 .75,0 1,0 .66,1 .35,1', '.40,.42 .50,.76 .61,.42'],
    I: ['0,0 .6,0 .6,.18 .42,.18 .42,.82 .6,.82 .6,1 0,1 0,.82 .18,.82 .18,.18 0,.18'],
    C: ['.18,0 1,0 1,.22 .31,.22 .25,.28 .25,.72 .31,.78 1,.78 1,1 .18,1 0,.82 0,.18'],
    F: ['0,0 .26,0 .26,.38 .81,.38 .81,.59 .26,.59 .26,.78 1,.78 1,1 0,1'],
    L: ['0,0 1,0 1,.22 .26,.22 .26,1 0,1'],
    O: ['.18,0 .82,0 1,.18 1,.82 .82,1 .18,1 0,.82 0,.18', '.32,.22 .25,.29 .25,.71 .32,.78 .68,.78 .75,.71 .75,.29 .68,.22'],
    W: ['.19,0 .43,0 .57,.57 .73,0 .97,0 1.17,1 .92,1 .82,.4 .67,1 .48,1 .32,.4 .23,1 0,1'],
}

function points(outline) {
    return outline.split(' ').map(pair => new Vector2(...pair.split(',').map(Number)))
}

export function installCampSign(model) {
    const letters = model.children.filter(child => child.name.startsWith('refLettersPhysicalDynamic'))
    letters.sort((left, right) => left.position.x - right.position.x)
    const word = 'STATICFLOW'
    if (letters.length !== word.length) throw new Error('Unexpected camp sign geometry')

    const direction = new Vector3().subVectors(letters.at(-1).position, letters[0].position).normalize()
    const middle = letters[0].position.clone().lerp(letters.at(-1).position, 0.5)
    const meshes = letters.map((mesh, index) => {
        const [border, ...holes] = outlines[word[index]]
        const shape = new Shape(points(border))
        shape.holes = holes.map(hole => new Path(points(hole)))
        const geometry = new ExtrudeGeometry(shape, {
            depth: 0.26, bevelEnabled: true, bevelSize: 0.012, bevelThickness: 0.025, bevelSegments: 1,
        })
        geometry.scale(1.28, 1.42, 1.42)
        geometry.center()
        geometry.computeBoundingBox()
        // Keep sampling the same swatch of the world palette as the old mesh.
        const palette = mesh.geometry.getAttribute('uv')
        const uv = geometry.getAttribute('uv')
        for (let i = 0; i < uv.count; i++) uv.setXY(i, palette.getX(0), palette.getY(0))
        mesh.geometry.dispose()
        mesh.geometry = geometry
        const size = geometry.boundingBox.getSize(new Vector3())
        const collider = mesh.children.find(child => child.name.startsWith('cuboid'))
        collider.scale.copy(size)
        collider.position.set(0, 0, 0)
        return { mesh, size }
    })

    const gap = 0.12
    let cursor = -(meshes.reduce((width, { size }) => width + size.x, 0) + gap * (meshes.length - 1)) / 2
    for (const { mesh, size } of meshes) {
        mesh.position.copy(middle).addScaledVector(direction, cursor + size.x / 2)
        cursor += size.x + gap
    }
}
