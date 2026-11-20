import assert from 'node:assert/strict'
import { createRequire, registerHooks } from 'node:module'
import test from 'node:test'
import { pathToFileURL } from 'node:url'

// The world installs its pinned Three dependency in its own build cache.
// NODE_PATH lets this source-level regression use that same installation.
let threeURL
try {
    threeURL = pathToFileURL(createRequire(import.meta.url).resolve('three/webgpu')).href
} catch (error) {
    if (error.code !== 'MODULE_NOT_FOUND') throw error
}

test('rear glass tracks visible instances as the world and vehicle move', {
    skip: !threeURL && 'Set NODE_PATH to the prepared world’s node_modules',
}, async t => {
    const hook = registerHooks({
        resolve(specifier, context, next) {
            if (specifier === 'three/webgpu') return { url: threeURL, shortCircuit: true }
            const resolved = next(specifier, context)
            if (specifier.endsWith('/rear-view-instances.js')) return { ...resolved, format: 'module' }
            return resolved
        },
    })
    t.after(() => hook.deregister())
    const { BoxGeometry, InstancedMesh, Matrix4, MeshBasicMaterial, PerspectiveCamera, Scene } = await import('three/webgpu')
    const { createRearViewInstances } = await import('../world/overlay/sources/staticflow/rear-view-instances.js')
    const scene = new Scene()
    const camera = new PerspectiveCamera(45, 1, .1, 30)
    const source = new InstancedMesh(new BoxGeometry(1, 1, 1), new MeshBasicMaterial(), 3)
    // The mesh's local origin is outside the camera. Instance zero is initially
    // off-screen, instance one is behind the vehicle, and slot two is dormant.
    source.position.x = 20
    const place = (index, x) => source.setMatrixAt(index, new Matrix4().makeTranslation(x, 0, -10))
    place(0, 0)
    place(1, -20)
    place(2, -19)
    source.count = 2
    scene.add(source)
    scene.updateMatrixWorld()
    const mirror = createRearViewInstances({ bushes: { foliage: { mesh: source } }, areas: {} }, scene)
    const draw = () => {
        const original = source.instanceMatrix.array.slice()
        const count = source.count
        camera.updateMatrixWorld()
        mirror.show(camera, object => { object.visible = false })
        const visible = []
        mirror.group.traverseVisible(object => {
            if (!object.isInstancedMesh) return
            for (let i = 0; i < object.count; i++) {
                const matrix = new Matrix4()
                object.getMatrixAt(i, matrix)
                visible.push(matrix.premultiply(object.matrixWorld).elements[12])
            }
        })
        assert.equal(source.visible, false)
        assert.equal(source.count, count)
        assert.deepEqual(source.instanceMatrix.array, original)
        source.visible = true
        mirror.group.visible = false
        return visible
    }
    assert.equal(mirror.group.visible, false)
    assert.deepEqual(draw(), [0])
    source.count = 3 // Enabling a populated slot does not dirty the matrix buffer.
    assert.deepEqual(draw(), [0, 1])
    place(0, -18)
    source.instanceMatrix.needsUpdate = true
    assert.deepEqual(draw(), [2, 0, 1])
    camera.position.x = 20
    assert.deepEqual(draw(), [])
    source.position.x += 20
    source.updateMatrixWorld()
    assert.deepEqual(draw(), [22, 20, 21])
    source.geometry = new BoxGeometry(1, 1, 1).translate(20, 0, 0)
    assert.deepEqual(draw(), [])
    camera.position.x += 20
    source.count = 1
    assert.deepEqual(draw(), [22])
})
