#version 300 es
precision highp float;

uniform vec2 viewport;
uniform float pixel_ratio;
uniform mat4 camera;
uniform vec3 anchors[6];
uniform vec4 aligned;
uniform float clock_s;
uniform int layer;
uniform int segments;

out vec3 ink;
out float edge;
out float along;
out float brightness;
flat out int material;
flat out float seed;

const float TAU = 6.2831853;
const vec3 COPPER = vec3(1.0, 0.44, 0.16);
const vec3 GOLD = vec3(1.0, 0.76, 0.40);
const vec3 ICE = vec3(0.28, 0.83, 0.85);

float hash(float n) { return fract(sin(n * 127.1 + 31.7) * 43758.5453); }
vec3 spine(float t) {
    float position = clamp(t, 0.0, 1.0) * 5.0;
    int index = min(int(position), 4);
    float part = position - float(index);
    return vec3(mix(anchors[index].x, anchors[index + 1].x, part),
                mix(anchors[index].yz, anchors[index + 1].yz, smoothstep(0.0, 1.0, part)));
}

// The same four bits that resolve the puzzle open or close the local braid.
float divergence_at(float t) {
    vec4 d = (vec4(0.2, 0.4, 0.6, 0.8) - t) * 10.0;
    return dot(1.0 - aligned, exp(-d * d));
}

vec3 line_position(float t, int id) {
    if (layer == 1) {
        float family = float(id / 8);
        float strand = (float(id % 8) - 3.5) / 3.5;
        float phase = family * TAU / 5.0 + 8.0 * t + 0.45 * sin(t * TAU)
                    + strand * 0.22 + 0.045 * sin(clock_s * 0.24 + family);
        float wave = sin(3.0 * TAU * t);
        float radius = sin(t * TAU * 0.5) * (0.22 + 0.71 * divergence_at(t)
                     + 0.19 * wave * wave) * (1.0 + 0.13 * strand);
        return spine(t) + vec3(0.0, radius * cos(phase), radius * sin(phase));
    }
    if (layer == 2) {
        int gate = id / 3;
        float band = float(id % 3);
        float radius = 0.87 + 0.105 * band + 0.12 * (1.0 - aligned[gate]);
        return anchors[gate + 1] + vec3(0.07 * (band - 1.0),
                          radius * cos(TAU * t), radius * sin(TAU * t));
    }
    // An etched ground plane gives orbit and parallax a stationary reference.
    if (id < 21) return vec3((float(id) - 10.0) * 0.47, -1.40, mix(-2.8, 2.8, t));
    return vec3(mix(-4.7, 4.7, t), -1.40, float(id - 27) * 0.45);
}

vec4 project(vec3 p) {
    return camera * vec4(p, 1.0);
}

void main() {
    if (layer == 0) {
        float n = float(gl_VertexID);
        vec3 p = (vec3(hash(n), hash(n + 11.0), hash(n + 83.0)) - 0.5)
               * vec3(13.2, 5.7, 5.0);
        gl_Position = project(p);
        gl_PointSize = (1.0 + 2.0 * hash(n + 17.0)) * pixel_ratio;
        ink = mix(ICE, GOLD, hash(n + 42.0));
        brightness = 0.19 + 0.48 * hash(n + 25.0);
        material = 0;
        edge = 0.0;
        along = 0.0;
        seed = n;
        return;
    }
    int id = gl_InstanceID;
    float t = float(gl_VertexID / 2) / float(segments);
    float side = gl_VertexID % 2 == 0 ? -1.0 : 1.0;
    vec4 clip = project(line_position(t, id));
    vec4 neighbor = project(line_position(t + (t < 1.0 ? 0.003 : -0.003), id));
    vec2 direction = (neighbor.xy / neighbor.w - clip.xy / clip.w) * viewport;
    if (t >= 1.0) direction *= -1.0;
    vec2 normal = vec2(-direction.y, direction.x) / max(length(direction), 0.00001);
    float width_px;
    seed = float(id);
    along = t;
    edge = side;
    if (layer == 1) {
        int family = id / 8;
        vec3 cool = family < 2 ? COPPER : (family == 2 ? ICE :
                       (family == 3 ? vec3(0.49, 0.56, 0.93) : vec3(0.68, 0.89, 0.83)));
        ink = mix(GOLD, cool, clamp(0.28 + 0.72 * divergence_at(t), 0.0, 1.0));
        brightness = 0.48 + 0.24 * hash(seed);
        width_px = (id % 8 == 3) ? 4.0 : 2.25;
        material = 1;
    } else if (layer == 2) {
        int band = id % 3;
        ink = mix(ICE, GOLD, aligned[id / 3]);
        brightness = band == 1 ? 0.33 : 0.76;
        width_px = band == 2 ? 3.6 : 2.6;
        material = band == 2 ? 3 : 2;
    } else {
        ink = vec3(0.35, 0.53, 0.53);
        brightness = 0.25;
        width_px = 1.0;
        material = 4;
    }
    clip.xy += normal * side * width_px * pixel_ratio * 2.0 / viewport * clip.w;
    gl_Position = clip;
}
