import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

// Three applies the mesh's instance matrix to both position and normal before
// the material's positionNode. This also lets a mirror batch share the material
// while using only the instance matrices inside its own field of view.
function nativeVegetationInstances(material) {
    return [/^    setInstancedMesh\(\)\n    \{[\s\S]*?^    \}/m, `    setInstancedMesh()
    {
        this.mesh = new THREE.InstancedMesh(this.geometry, ${material}, this.transformMatrices.length)
        this.transformMatrices.forEach((matrix, index) => this.mesh.setMatrixAt(index, matrix))
        this.mesh.instanceMatrix.needsUpdate = true
        this.mesh.castShadow = true
        this.mesh.receiveShadow = true
        this.mesh.frustumCulled = false
        this.game.scene.add(this.mesh)
    }`]
}

export default async function adaptCamera(root) {
    const files = [
        ['Rendering.js', [
            ['        this.postProcessing.render()', '        this.postProcessing.render()\n        this.game.view.firstPerson.updateMirror()'],
        ]],
        ['World/Areas/CircuitArea.js', [
            ['PI, positionGeometry, positionWorld', 'PI, positionGeometry, positionView, positionWorld'],
            // A checkpoint is a marker on the road. Fade it out before the
            // driver passes through its plane and its stripes cross the cab.
            ['baseUv.y.oneMinus().mul(stripes)', 'baseUv.y.oneMinus().mul(stripes).mul(positionView.z.negate().smoothstep(1.05, 4))'],
            ['material.outputNode = doorOutputColor(this.checkpoints.doorTarget', 'material.depthWrite = false\n            material.outputNode = doorOutputColor(this.checkpoints.doorTarget'],
            ['material.outputNode = doorOutputColor(this.checkpoints.doorReached', 'material.depthWrite = false\n            material.outputNode = doorOutputColor(this.checkpoints.doorReached'],
        ]],
        ['View.js', [
            ["import CameraControls", "import { DriverView } from '../staticflow/driver-view.js'\nimport CameraControls"],
            ['        this.setMapControls()', '        this.firstPerson = new DriverView(this)\n        this.setMapControls()'],
            ['        // Cameras matrices', '        this.firstPerson.updatePose()\n\n        // Cameras matrices'],
            ['        // Helper\n        if(this.optimalArea.quad2Helper', '        this.firstPerson.updateFootprint()\n\n        // Helper\n        if(this.optimalArea.quad2Helper'],
            ["this.game.inputs.events.on('viewMapPointer', (action) =>\n        {", "this.game.inputs.events.on('viewMapPointer', (action) =>\n        {\n            if(this.firstPerson.enabled) return"],
            ['this.game.inputs.gamepad.joysticks.right.active && !this.cinematic.active', 'this.game.inputs.gamepad.joysticks.right.active && !this.cinematic.active && !this.firstPerson.enabled'],
            ['this.speedLines.clipSpaceTarget.value.project(this.camera)', `if (this.firstPerson.weight > 0)
            this.speedLines.clipSpaceTarget.value.set(0, 0, 0)
        else
            this.speedLines.clipSpaceTarget.value.project(this.camera)`],
        ]],
        ['Fog.js', [
            ['this.game.dayCycles.properties.fogFarRatio.value * amplitude', `this.game.dayCycles.properties.fogFarRatio.value * amplitude
        const view = this.game.view
        if (view.camera.aspect < 1 && view.focusPoint.isTracking && !view.freeMode.enabled && !view.cinematic.progress) {
            // The portrait chase camera pulls back to frame the road. Keep the
            // vehicle within the first quarter of the fog ramp (16% opacity),
            // even at night. The far shore still fades into the sky.
            const carDepth = view.spherical.radius.current
            const depthToHorizon = this.far.value - carDepth
            if (depthToHorizon > 0) this.near.value = Math.max(this.near.value, carDepth - depthToHorizon / 3)
        }
        // The overview fog distances describe the slant range from above.
        // At eye level, blend the horizon into fog inside the streamed floor.
        const weight = view.firstPerson.weight
        if (weight > 0) {
            const radius = this.game.view.optimalArea.radius
            const nearRatio = this.game.dayCycles.properties.fogNearRatio.value
            this.near.value += (radius * (0.40 + nearRatio * 0.18) - this.near.value) * weight
            this.far.value += (radius * 1.12 - this.far.value) * weight
        }`],
        ]],
        ['World/Foliage.js', [
            ["import { alea }", "import { driverFoliage } from '../../staticflow/driver-view.js'\nimport { alea }"],
            ['Fn, instance, mix', 'Fn, mix'],
            [/^        \/\/ Position\n        this\.material\.instance\.positionNode = Fn\([\s\S]*?^        \}\)\(\)\n\n/m, ''],
            nativeVegetationInstances('this.material.instance'),
            ['        this.material.seeThroughPosition.value.copy(this.game.world.visualVehicle.screenPosition)', `        if (this.game.view.firstPerson.weight > .5) {
            this.driverGeometry ??= driverFoliage(this.geometry)
            this.mesh.geometry = this.driverGeometry
            // The driver looks through the windscreen. The overhead cutout
            // around the projected vehicle would erase foliage on the road.
            this.material.seeThroughPosition.value.set(-10, -10)
        } else {
            this.mesh.geometry = this.geometry
            this.material.seeThroughPosition.value.copy(this.game.world.visualVehicle.screenPosition)
        }`],
        ]],
        ['World/Flowers.js', [
            [/^import \{[^\n]+\} from 'three\/tsl'/m, "import { color, positionLocal, uniform, vec3 } from 'three/tsl'"],
            [/^        this\.material\.positionNode = Fn\([\s\S]*?^        \}\)\(\)/m,
                '        this.material.positionNode = positionLocal.add(vec3(wind.x, 0, wind.y).mul(multiplier))'],
            nativeVegetationInstances('this.material'),
        ]],
        ['World/VisualVehicle.js', [
            ['        this.parts.chassis.position.copy(physicalVehicle.position)', `        const showExterior = !this.game.view.firstPerson.hideExterior
        this.parts.chassis.visible = showExterior
        if (this.antenna) this.antenna.head.visible = showExterior
        this.parts.chassis.position.copy(physicalVehicle.position)`],
        ]],
        ['Physics/PhysicsVehicle.js', [
            ['        this.position.copy(position)\n    }\n\n    updatePrePhysics()', `        this.position.copy(position)
        this.events.trigger('teleport')
    }

    updatePrePhysics()`],
        ]],
        ['Passes/cheapDOF.js', [
            ['this.size = uniform(2)', 'this.size = uniform(2)\n        this.strength = uniform(1.5)\n        this.overview = uniform(1)'],
            ['.smoothstep(this.start, this.end)', '.smoothstep(this.start, this.end).mul(this.strength.div(1.5)).mul(this.overview)'],
        ]],
        ['Inputs/Nipple.js', [
            ['                    // Intersect', `                    if (this.game.view.firstPerson.weight > 0) {
                        this.game.view.firstPerson.updateTouch(this, pointer, action.trigger === 'start')
                        return
                    }
                    // Intersect`],
            ['            // Smallest angle and forward', `            if (this.active && this.game.view.firstPerson.weight > 0)
                this.targetAngle = this.angle + this.game.view.firstPerson.touchAngle
            // Smallest angle and forward`],
            ['            this.group.visible = true', '            this.group.visible = !this.game.view.firstPerson.weight'],
        ]],
        ['InteractivePoints.js', [
            ['        item.position = new THREE.Vector2', '        item.group = group\n        item.position = new THREE.Vector2'],
            ['        // Player testing (not cursor intersect)', `        const driverView = this.game.view.firstPerson.weight > 0
        if (driverView || this.wasDriverView) {
            for (const item of this.items) {
                if (driverView && item.state !== InteractivePoints.STATE_HIDDEN) item.group.quaternion.copy(this.game.view.camera.quaternion)
                else if (!driverView) item.group.rotation.set(-Math.PI * .15, Math.PI * .25, 0, 'YXZ')
            }
        }
        this.wasDriverView = driverView
        // Player testing (not cursor intersect)`],
        ]],
    ]
    for (const [file, edits] of files) {
        const path = join(root, 'sources/Game', file)
        let source = await readFile(path, 'utf8')
        for (const [anchor, replacement] of edits) {
            const found = typeof anchor === 'string' ? source.includes(anchor) : anchor.test(source)
            if (!found) throw new Error('Missing driver camera integration in ' + file + ': ' + anchor)
            source = source.replace(anchor, replacement)
        }
        await writeFile(path, source)
    }
}
