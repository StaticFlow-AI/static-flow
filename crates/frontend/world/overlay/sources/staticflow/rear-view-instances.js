import { DynamicDrawUsage, Frustum, Group, InstancedMesh, Matrix4, Sphere } from 'three/webgpu'

// The world batches repeated objects across the whole island. Three can reject
// an entire InstancedMesh, but a single visible tree otherwise submits every
// tree. Pack just the instances inside the mirror's frustum into a second draw
// buffer. Geometry, material, and the simulation's original matrices are shared
// or read-only; the dashboard never uploads a changed buffer to the main view.
export function createRearViewInstances(world, scene) {
    const group = new Group()
    group.visible = false
    scene.add(group)
    const batches = []
    const matrix = new Matrix4()
    const clipFromWorld = new Matrix4()
    const frustum = new Frustum()

    function add(source, windMargin = 0) {
        const mesh = new InstancedMesh(source.geometry, source.material, source.instanceMatrix.count)
        mesh.name = source.name
        mesh.instanceMatrix.setUsage(DynamicDrawUsage)
        mesh.receiveShadow = source.receiveShadow
        mesh.renderOrder = source.renderOrder
        mesh.frustumCulled = false
        mesh.matrixAutoUpdate = false
        mesh.matrixWorldAutoUpdate = false
        mesh.count = 0
        group.add(mesh)
        batches.push({
            source, mesh, windMargin, version: -1, sourceCount: 0,
            bounds: Array.from({ length: source.instanceMatrix.count }, () => new Sphere()),
            indices: new Uint32Array(source.instanceMatrix.count),
        })
    }

    for (const tree of [world.birchTrees, world.oakTrees, world.cherryTrees]) {
        if (tree) {
            add(tree.bodies)
            add(tree.leaves.mesh)
        }
    }
    if (world.bushes) add(world.bushes.foliage.mesh)
    if (world.flowers) add(world.flowers.mesh, 1) // Maximum wind displacement in world units.
    for (const set of [world.bricks, world.benches, world.fences, world.poleLights, world.lanterns,
        world.explosiveCrates, world.areas.cookie?.cookies]) {
        if (set) for (const { instance } of set.instancedGroup.meshes) add(instance)
    }

    function show(camera, hide) {
        clipFromWorld.multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse)
        group.visible = true
        for (const batch of batches) {
            const { source, mesh, bounds, indices } = batch
            mesh.visible = source.visible
            if (!source.visible) continue
            hide(source)
            mesh.material = source.material
            mesh.matrixWorld.copy(source.matrixWorld)
            // Test bounds in the coordinate system of the source batch.
            matrix.multiplyMatrices(clipFromWorld, source.matrixWorld)
            frustum.setFromProjectionMatrix(matrix, camera.coordinateSystem, camera.reversedDepth)
            const changed = batch.version !== source.instanceMatrix.version ||
                mesh.geometry !== source.geometry || batch.sourceCount !== source.count
            if (changed) {
                mesh.geometry = source.geometry
                if (!mesh.geometry.boundingSphere) mesh.geometry.computeBoundingSphere()
                for (let i = 0; i < source.count; i++) {
                    source.getMatrixAt(i, matrix)
                    bounds[i].copy(mesh.geometry.boundingSphere).applyMatrix4(matrix)
                    bounds[i].radius += batch.windMargin
                }
                batch.version = source.instanceMatrix.version
                batch.sourceCount = source.count
            }

            let count = 0
            let upload = changed
            for (let i = 0; i < source.count; i++) {
                if (!frustum.intersectsSphere(bounds[i])) continue
                if (indices[count] !== i) upload = true
                indices[count++] = i
            }
            mesh.visible = count > 0
            if (count && (upload || count > mesh.count)) {
                for (let i = 0; i < count; i++) {
                    source.getMatrixAt(indices[i], matrix)
                    mesh.setMatrixAt(i, matrix)
                }
                mesh.instanceMatrix.clearUpdateRanges()
                mesh.instanceMatrix.addUpdateRange(0, count * 16)
                mesh.instanceMatrix.needsUpdate = true
            }
            mesh.count = count
        }
    }
    return { group, show }
}
