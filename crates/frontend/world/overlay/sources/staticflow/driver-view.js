import { MathUtils, Quaternion, Vector2, Vector3 } from 'three/webgpu'
import { createDriverCockpit } from './driver-cockpit.js'

export class DriverView {
    constructor(view) {
        this.view = view
        this.enabled = false
        this.weight = 0
        this.cockpit = null
        this.steeringAngle = 0
        this.lastSteeringTime = null
        this.hideExterior = false
        this.eye = new Vector3()
        this.heading = new Vector3(1, 0, 0)
        this.target = new Vector3()
        this.touchOrigin = new Vector2()
        this.touchAngle = 0
        this.dialogOpen = false
        document.addEventListener('toggle', event => {
            if (event.target.tagName === 'DIALOG') this.dialogOpen = Boolean(document.querySelector('dialog[open]'))
        }, true)
    }

    setEnabled(enabled) {
        this.enabled = enabled
        this.lastSteeringTime = null
        if (enabled) this.view.focusPoint.isTracking = true
        const nipple = this.view.game.inputs.nipple
        nipple.active = false
        nipple.inRadiusLow = false
        nipple.progress = 0
    }

    updatePose() {
        const { camera, defaultCamera, cinematic, game, focusPoint } = this.view
        const car = game.physicalVehicle
        const filters = game.inputs.filters
        const panelOpen = this.dialogOpen || filters.has('modal') || filters.has('menu')
        // The launch animation enables driving while its reveal shader still
        // masks the road. Keep the driver's selection until the grid is gone.
        const ready = game.reveal?.step === 2 && focusPoint.isTracking && !this.view.freeMode.enabled && !panelOpen
        this.weight = this.enabled && car && ready ? 1 - cinematic.progress : 0
        // Use the existing render-pass camera. Cursor picking, audio and the
        // world all read its exact pose. Keep the independent overview camera
        // intact for teleporting, resizing and returning from an exhibit.
        if (this.weight > 0) {
            const forward = car.forward
            if (Math.hypot(forward.x, forward.z) > .02)
                this.heading.set(forward.x, 0, forward.z).normalize()
            const pitch = MathUtils.clamp(forward.y, -.42, .42) - .09
            // The chassis uses +X forward and -Z for the left seat. Attach
            // the eye to its rigid body, including on jumps and rollovers.
            this.eye.set(-.06, .43, -.27).applyQuaternion(car.quaternion).add(car.position)
            camera.position.copy(this.eye)
            this.target.copy(camera.position).add(this.heading)
            this.target.y += pitch
            camera.lookAt(this.target)
            // Feel the suspension when crossing a curb without rolling the
            // entire horizon on a hard collision.
            const bank = Math.atan2(car.sideward.y, car.upward.y)
            camera.rotateZ(MathUtils.clamp(bank * .6, -.34, .34))
            if (cinematic.progress > 0) {
                camera.position.lerp(cinematic.dummy.position, cinematic.progress)
                camera.quaternion.slerp(cinematic.dummy.quaternion, cinematic.progress)
            }
        }
        if (game.rendering?.cheapDOFPass) game.rendering.cheapDOFPass.overview.value = 1 - this.weight
        const fov = MathUtils.lerp(defaultCamera.fov, camera.aspect < 1 ? 78 : 68, this.weight)
        if (camera.fov !== fov) {
            camera.fov = fov
            camera.updateProjectionMatrix()
        }

        // During an exhibit cutaway the camera travels outside the vehicle.
        // Let the dashboard leave with the seat; reveal the opaque exterior
        // once the moving camera has cleared the cab.
        const eyeDistanceSq = this.weight > 0 ? camera.position.distanceToSquared(this.eye) : Infinity
        this.hideExterior = eyeDistanceSq < 1.2 * 1.2
        const seated = eyeDistanceSq < .06 * .06
        if (seated) {
            if (!this.cockpit) {
                this.cockpit = createDriverCockpit()
                camera.add(this.cockpit.group)
                car.events.on('teleport', () => { this.lastSteeringTime = null })
            }
            const targetAngle = car.controller.wheelSteering(0)
            const now = game.ticker.elapsed
            // Rapier applies a keyboard turn in one step. Give the hands a
            // short physical response using real elapsed seconds; the physics
            // ticker separately caps delta and scales simulation time.
            this.steeringAngle = this.lastSteeringTime === null ? targetAngle :
                MathUtils.damp(this.steeringAngle, targetAngle, 1 / .085, now - this.lastSteeringTime)
            if (Math.abs(this.steeringAngle - targetAngle) < .0005) this.steeringAngle = targetAngle
            this.lastSteeringTime = now
            this.cockpit.update(this.steeringAngle, camera.aspect, camera.fov)
        } else this.lastSteeringTime = null
        if (this.cockpit) this.cockpit.group.visible = seated
    }

    updateFootprint() {
        if (!this.weight) return
        const { optimalArea: area, game } = this.view
        // Keep the same streamed ground/grass/shadow tile sizes. Move their
        // center toward the windscreen so the road is present up to the fog.
        const radius = area.radius
        const car = game.physicalVehicle.position
        this.target.set(car.x + this.heading.x * radius * .44, 0, car.z + this.heading.z * radius * .44)
        area.position.lerp(this.target, this.weight)
        for (let i = 0; i < 4; i++) {
            const corner = area.quad2[i].offseted
            corner.x = MathUtils.lerp(corner.x, this.target.x + (i < 2 ? radius : -radius), this.weight)
            corner.y = MathUtils.lerp(corner.y, this.target.z + (i === 0 || i === 3 ? radius : -radius), this.weight)
        }
    }

    updateTouch(nipple, pointer, start) {
        if (start) this.touchOrigin.set(pointer.current.x, pointer.current.y)
        const dx = pointer.current.x - this.touchOrigin.x
        const dy = this.touchOrigin.y - pointer.current.y
        const distance = Math.hypot(dx, dy)
        const travel = MathUtils.clamp(Math.min(this.view.game.viewport.width, this.view.game.viewport.height) * .14, 45, 80)
        nipple.progress = MathUtils.clamp((distance - 5) / travel, 0, 1)
        nipple.uniforms.progress.value = nipple.progress
        if (start) nipple.inRadiusLow = true
        else if (nipple.progress > 0) nipple.inRadiusLow = false
        // The same screen-space drag continues to steer relative to the
        // windscreen as the car turns. Dragging back selects reverse.
        this.touchAngle = Math.atan2(dx, dy)
    }
}

// The overview foliage is made of parallel textured quads. A driver can see
// their backs or edges, so orient a second geometry around the same canopy.
// It shares the material and instance transforms and uses one draw per grove.
export function driverFoliage(overview) {
    const geometry = overview.clone()
    const vertices = geometry.getAttribute('position')
    const leafCenter = new Vector3()
    const corner = new Vector3()
    const face = new Vector3(0, 0, 1)
    const rotation = new Quaternion()
    for (let i = 0; i < vertices.count; i += 4) {
        leafCenter.fromBufferAttribute(vertices, i).add(corner.fromBufferAttribute(vertices, i + 3)).multiplyScalar(.5)
        rotation.setFromUnitVectors(face, corner.copy(leafCenter).normalize())
        for (let j = i; j < i + 4; j++) {
            corner.fromBufferAttribute(vertices, j).sub(leafCenter).applyQuaternion(rotation).add(leafCenter)
            vertices.setXYZ(j, corner.x, corner.y, corner.z)
        }
    }
    return geometry
}
