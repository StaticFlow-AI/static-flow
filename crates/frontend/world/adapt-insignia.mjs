import { readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const artwork = {
    circuitLogoWebgl: 'archive',
    circuitLogoWebgpu: 'radio',
    circuitLogoThreejs: 'worldline',
    circuitBrand: 'staticflow',
    projectsCarpet: 'archive-pennant',
    projectsLabels: 'projects-labels',
    labCarpet: 'lab-pennant',
    cookieBanner: 'oven-pennant',
}

const padded = size => Math.ceil(size / 4) * 4

// Replace the embedded images for every instance (including waving banners and
// the podium). Keep Draco geometry, original UVs, and unrelated KTX2 data byte-exact.
async function replaceInsignia(file, decals) {
    const original = await readFile(file)
    const endOfJson = 20 + original.readUInt32LE(12)
    const gltf = JSON.parse(original.toString('utf8', 20, endOfJson))
    const binary = original.subarray(endOfJson + 8)
    const replacementViews = new Map()
    const replacedImages = new Set()

    for (const [name, image] of decals) {
        const index = gltf.images.findIndex(entry => entry.name === name)
        if (index < 0) throw new Error(`Missing scene insignia ${name} in ${file}`)
        const item = gltf.images[index]
        item.name = artwork[name]
        item.mimeType = 'image/png'
        replacedImages.add(index)
        replacementViews.set(item.bufferView, image)
    }

    for (const texture of gltf.textures) {
        const source = texture.extensions?.KHR_texture_basisu?.source
        if (replacedImages.has(source)) {
            texture.source = source
            delete texture.extensions.KHR_texture_basisu
            if (!Object.keys(texture.extensions).length) delete texture.extensions
        }
    }

    const chunks = []
    let offset = 0
    for (const [index, view] of gltf.bufferViews.entries()) {
        const start = view.byteOffset ?? 0
        const data = replacementViews.get(index) ?? binary.subarray(start, start + view.byteLength)
        chunks.push(data, Buffer.alloc(padded(data.length) - data.length))
        view.byteOffset = offset
        view.byteLength = data.length
        offset += padded(data.length)
    }
    gltf.buffers[0].byteLength = offset

    const json = Buffer.from(JSON.stringify(gltf))
    const header = Buffer.from(original.subarray(0, 20))
    const binaryHeader = Buffer.from(original.subarray(endOfJson, endOfJson + 8))
    header.writeUInt32LE(28 + padded(json.length) + offset, 8)
    header.writeUInt32LE(padded(json.length), 12)
    binaryHeader.writeUInt32LE(offset, 0)
    await writeFile(file, Buffer.concat([header, json, Buffer.alloc(padded(json.length) - json.length, 0x20), binaryHeader, ...chunks]))
}

export default async function adaptInsignia(root) {
    const decals = await Promise.all(Object.entries(artwork).map(async ([name, image]) =>
        [name, await readFile(new URL(`./insignia/${image}.png`, import.meta.url))]))
    for (const filename of ['areas.glb', 'areas-compressed.glb']) {
        await replaceInsignia(join(root, 'static/areas', filename), decals)
    }

    // Keep national flags in the score editor and substitute blog pennants for
    // the upstream portfolio's fictional/sponsor flags.
    const countries = join(root, 'sources/data/countries.js')
    const source = await readFile(countries, 'utf8')
    const start = source.indexOf("    [ 'Luffy',")
    if (start < 0 || !source.includes("'Three.js Journey', 'jj'")) throw new Error('Unexpected racing flag catalog')
    await writeFile(countries, source.slice(0, start) + [
        "    [ '档案站', 'StaticFlow Archive', 'wl' ],",
        "    [ '电台', 'StaticFlow Radio', 'wu' ],",
        "    [ '世界线', 'StaticFlow Worldline', 'ts' ],",
        ']',
    ].join('\n'))

    // Scores, whispers and saved preferences may still carry a retired flag
    // code. Resolve those with the same lookup as current entries, while the
    // editor only offers the three printable pennants. The API uses two letters.
    const editor = join(root, 'sources/Game/InputFlag.js')
    const input = await readFile(editor, 'utf8')
    if (!input.includes('this.setCountries()') || !input.includes('this.countries.forEach')) {
        throw new Error('Cannot adapt racing flag selection')
    }
    await writeFile(editor, input.replace('this.setCountries()', `this.setCountries()
        this.selectableCountries = [...this.countries.values()]
        for (const [legacy, current] of Object.entries({lf: 'wl', hy: 'ts', ub: 'ts', kj: 'wl', nv: 'wu', jj: 'wl'}))
            this.countries.set(legacy, this.countries.get(current))`)
        .replaceAll('this.countries.forEach', 'this.selectableCountries.forEach'))
}
