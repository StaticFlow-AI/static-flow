import './minimap.css'

const storageKey = 'staticflow.world.minimap'
const zoom = 2.25
const clamp = (value, min, max) => Math.min(max, Math.max(min, value))
const labels = { altar: '世界线观测站', projects: '文章工坊', social: '广播会馆', landing: '未来道具实验室' }

// Quarter-pixel positioning stops suspension jitter from repainting a parked car.
function place(sprite, x, y, angle = 0) {
    x = Math.round(x * 4) / 4
    y = Math.round(y * 4) / 4
    if (sprite.x === x && sprite.y === y && sprite.angle === angle) return
    sprite.x = x
    sprite.y = y
    sprite.angle = angle
    sprite.element.style.transform = `translate3d(${x}px,${y}px,0) rotate(${angle}rad)`
}

export class WorldMinimap {
    constructor(map) {
        this.map = map
        this.game = map.game
        this.root = document.createElement('aside')
        this.root.id = 'world-minimap'
        this.root.setAttribute('aria-label', '小地图')
        this.root.innerHTML = `
            <button type="button" data-minimap-toggle aria-controls="world-minimap-view">
                <i aria-hidden="true"></i>
            </button>
            <button type="button" id="world-minimap-view" data-minimap-open aria-label="打开完整地图">
                <span class="minimap-field">
                    <img data-minimap-terrain decoding="async" alt="" draggable="false">
                    <span class="minimap-north" aria-hidden="true">N</span>
                    <i data-minimap-goal hidden></i>
                    <img data-minimap-player src="ui/minimap/player.webp" alt="" draggable="false">
                </span>
            </button>`
        this.game.domElement.append(this.root)
        this.toggle = this.root.querySelector('[data-minimap-toggle]')
        this.view = this.root.querySelector('[data-minimap-open]')
        this.field = this.root.querySelector('.minimap-field')
        this.terrain = { element: this.root.querySelector('[data-minimap-terrain]') }
        this.terrain.element.style.width = (zoom * 100) + '%'
        this.player = { element: this.root.querySelector('[data-minimap-player]') }
        this.goal = { element: this.root.querySelector('[data-minimap-goal]') }

        let expanded = true
        try { expanded = localStorage.getItem(storageKey) !== 'hidden' } catch { /* Current visit only. */ }
        this.setExpanded(expanded)
        this.toggle.addEventListener('click', event => {
            this.setExpanded(this.view.hidden)
            if (event.detail) window.dispatchEvent(new Event('world-driving-focus'))
            try { localStorage.setItem(storageKey, this.view.hidden ? 'hidden' : 'visible') } catch { /* Current visit only. */ }
        })
        this.view.addEventListener('click', () => this.game.modals.open('map'))
        this.sample = () => {
            if (this.game.ticker.elapsed < this.nextSample) return
            this.nextSample = this.game.ticker.elapsed + .08
            this.update()
        }
        // The field also becomes zero-sized when the input mode or a dialog
        // hides the HUD, so covered and collapsed maps have no tick listener.
        const resize = new ResizeObserver(([entry]) => {
            this.size = entry.contentRect.width
            this.lastX = undefined
            this.syncActivity()
        })
        resize.observe(this.field)
        document.addEventListener('visibilitychange', () => this.syncActivity())
        this.game.dayCycles.events.on('night', () => {
            if (this.active) this.syncTexture()
        })
    }

    syncActivity() {
        const active = this.size > 0 && !this.view.hidden && !document.hidden
        if (active === this.active) return
        this.active = active
        if (active) {
            this.syncTexture()
            this.nextSample = 0
            this.lastX = undefined
            this.game.ticker.events.on('tick', this.sample, 14)
        } else {
            this.game.ticker.events.off('tick', this.sample)
        }
    }

    syncTexture() {
        const texture = this.map.textureUrl('ui/minimap')
        if (this.texture === texture) return
        this.texture = texture
        const image = this.terrain.element
        image.srcset = `${texture} 1x, ${texture.replace('.webp', '@2x.webp')} 2x`
        image.src = texture
    }

    setExpanded(expanded) {
        this.view.hidden = !expanded
        this.toggle.setAttribute('aria-expanded', String(expanded))
        this.toggle.setAttribute('aria-label', expanded ? '收起小地图' : '展开小地图')
        this.toggle.title = expanded ? '收起小地图' : '展开小地图'
        if (this.sample) this.syncActivity()
    }

    update() {
        const { game, map } = this
        const { position } = game.player
        const angle = Math.round(-game.physicalVehicle.yRotation * 360 / Math.PI) * (Math.PI / 360)
        const { route, origin: observatory } = game.world.areas.altar
        const target = route.target
        const complete = route.progress.active === 'gate'
        const goalChanged = target !== this.target || complete !== this.complete
        const mapWidth = this.size * zoom
        const tolerance = .25 * game.terrain.size / mapWidth
        const moved = this.lastX === undefined || Math.abs(position.x - this.lastX) >= tolerance ||
            Math.abs(position.z - this.lastZ) >= tolerance
        if (!moved && !goalChanged && angle === this.player.angle) return

        if (moved) {
            this.lastX = position.x
            this.lastZ = position.z
            const p = map.worldToMap(position)
            // At the coast the chart stops scrolling and the vehicle moves off-center.
            this.left = clamp(p.x * mapWidth - this.size / 2, 0, mapWidth - this.size)
            this.top = clamp(p.y * mapWidth - this.size / 2, 0, mapWidth - this.size)
            this.playerX = p.x * mapWidth - this.left
            this.playerY = p.y * mapWidth - this.top
            place(this.terrain, -this.left, -this.top)
        }
        place(this.player, this.playerX, this.playerY, angle)
        if (goalChanged) {
            this.target = target
            this.complete = complete
            this.goal.element.hidden = complete
            this.root.dataset.signal = target || 'observatory'
            const name = labels[route.signal]
            this.view.title = complete ? '打开完整地图' : `打开完整地图 · ${name}`
            this.view.setAttribute('aria-label', this.view.title)
            this.destination = complete ? null : map.worldToMap(route.stations[target]?.base || observatory)
        }
        if (!this.destination || (!moved && !goalChanged)) return
        const x = (this.destination.x * mapWidth - this.left) / this.size
        const y = (this.destination.y * mapWidth - this.top) / this.size
        // Stop a distant signal at the frame edge on the actual bearing from the car.
        const fromX = clamp(this.playerX / this.size, .06, .94)
        const fromY = clamp(this.playerY / this.size, .08, .92)
        const dx = x - fromX
        const dy = y - fromY
        const edgeX = dx < 0 ? .06 : .94
        const edgeY = dy < 0 ? .08 : .92
        const distance = Math.min(1, dx ? (edgeX - fromX) / dx : 1, dy ? (edgeY - fromY) / dy : 1)
        const distant = distance < 1
        if (distant !== this.distant) {
            this.distant = distant
            this.goal.element.classList.toggle('is-distant', distant)
        }
        const bearing = distant ? Math.round(Math.atan2(dy, dx) * 180 / Math.PI) * (Math.PI / 180) : Math.PI / 4
        place(this.goal, (fromX + distance * dx) * this.size, (fromY + distance * dy) * this.size, bearing)
    }
}
