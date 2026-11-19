import { Color, DoubleSide, MeshLambertMaterial, Vector3 } from 'three';
import { sampleAtlas } from './atlas.js';

// Voxel faces have constant normals. Evaluate diffuse lighting at their vertices and interpolate the
// irradiance. Only the sun visibility test depends on the pixel's world position.
export function voxelMaterials(texture) {
  const sunDirection = { value: new Vector3() },
    sun = { value: new Color() },
    ambient = { value: 1 },
    waterTime = { value: 0 };
  function compile(shader) {
    shader.uniforms.voxelSunDirection = sunDirection;
    shader.uniforms.voxelSun = sun;
    shader.uniforms.voxelAmbient = ambient;
    shader.vertexShader =
      `attribute vec3 irradiance;
      uniform vec3 voxelSunDirection,voxelSun; uniform float voxelAmbient;
      varying vec3 vFill,vSun;
    ` +
      shader.vertexShader.replace(
        '#include <begin_vertex>',
        `#include <begin_vertex>
      vec3 hemisphere=mix(vec3(.68,.71,.73),vec3(.98,1.02,1.09),normal.y*.5+.5);
      vFill=voxelAmbient*hemisphere*mix(.38,.96,irradiance.x)
        +vec3(.93,.65,.39)*irradiance.y*irradiance.y
        +vec3(.95,.84,.65)*irradiance.z;
      vSun=voxelSun*max(dot(normal,voxelSunDirection),0.)*smoothstep(.05,.75,irradiance.x);
    `
      );
    shader.fragmentShader = `
      uniform vec3 diffuse; uniform float opacity; varying vec3 vFill,vSun;
      #include <common>
      #include <uv_pars_fragment>
      #include <map_pars_fragment>
      #include <color_pars_fragment>
      #include <alphatest_pars_fragment>
      #include <fog_pars_fragment>
      #include <shadowmap_pars_fragment>
      void main() {
        vec4 diffuseColor=vec4(diffuse,opacity);
        #include <map_fragment>
        #include <color_fragment>
        #include <alphatest_fragment>
        float visibility=1.;
        #if defined(USE_SHADOWMAP) && NUM_DIR_LIGHT_SHADOWS > 0
          vec3 sc=vDirectionalShadowCoord[0].xyz/vDirectionalShadowCoord[0].w;
          if(all(greaterThanEqual(sc,vec3(0.))) && all(lessThanEqual(sc,vec3(1.)))) {
            sc.z+=directionalLightShadows[0].shadowBias;
            // One hardware-filtered 2x2 PCF footprint stays anchored to the world on camera motion.
            visibility=texture(directionalShadowMap[0],sc);
          }
        #endif
        vec3 outgoingLight=diffuseColor.rgb*(vFill+visibility*vSun);
        #include <opaque_fragment>
        #include <colorspace_fragment>
        #include <fog_fragment>
      }`;
    sampleAtlas(shader);
  }
  const materials = [0, 0.4].map((alphaTest) => {
    const m = new MeshLambertMaterial({ map: texture, vertexColors: true, alphaTest });
    m.onBeforeCompile = compile;
    return m;
  });
  const water = new MeshLambertMaterial({
    map: texture,
    vertexColors: true,
    transparent: true,
    opacity: 0.76,
    side: DoubleSide,
    depthWrite: false
  });
  water.forceSinglePass = true;
  water.onBeforeCompile = (shader) => {
    compile(shader);
    shader.uniforms.waterTime = waterTime;
    const varyings = 'varying vec3 waterPosition, waterNormal;\n';
    shader.vertexShader =
      varyings +
      shader.vertexShader.replace(
        '#include <begin_vertex>',
        `#include <begin_vertex>
       waterPosition=(modelMatrix*vec4(position,1.)).xyz;
       waterNormal=normal;`
      );
    shader.fragmentShader =
      varyings +
      'uniform float waterTime;\n' +
      shader.fragmentShader.replace(
        '#include <opaque_fragment>',
        `vec3 n=normalize(waterNormal);
       bool surface=abs(n.y)>.5;
       vec2 p=surface ? waterPosition.xz : vec2(waterPosition.x+waterPosition.z,-waterPosition.y);
       // The tilted surface normal points downstream on a slope. Vertical
       // faces use falling streaks, and a level source only ripples in place.
       vec2 current=surface ? n.xz/max(.20,length(n.xz))*.48 : vec2(0.,.95);
       p-=current*waterTime;
       float ripple=sin(p.x*4.1+p.y*2.9+sin(p.y*3.3+waterTime*.8))
                   +sin(p.y*4.8-p.x*1.6+waterTime*.65);
       float glint=smoothstep(1.42,1.94,ripple);
       outgoingLight+=vec3(.075,.14,.15)*glint*(.25+vFill.y);
       #include <opaque_fragment>`
      );
  };
  materials.push(water);
  function update(direction, strength, skylight, time = 0) {
    sunDirection.value.copy(direction);
    sun.value.setRGB(1, 0.82, 0.61).multiplyScalar(strength);
    ambient.value = skylight;
    waterTime.value = time;
  }
  return { materials, update };
}
