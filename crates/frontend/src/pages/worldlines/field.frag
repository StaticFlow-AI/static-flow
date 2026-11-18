#version 300 es
precision highp float;
uniform float clock_s;
uniform vec2 transmission;
in vec3 ink;
in float edge;
in float along;
in float brightness;
flat in int material;
flat in float seed;
out vec4 color;

void main() {
    float light = brightness;
    vec3 tint = ink;
    if (material == 0) {
        light *= 1.0 - smoothstep(0.08, 0.50, length(gl_PointCoord - 0.5));
    } else {
        float d = abs(edge);
        light *= 0.68 * exp(-6.5 * d * d) + 0.32 * (1.0 - d);
        if (material == 1) {
            float distance_to_packet = fract(along - clock_s * 0.115 + floor(seed / 8.0) * 0.21);
            float pulse = exp(-55.0 * distance_to_packet);
            if (transmission.y < 1.2) {
                float wave = (abs(along - transmission.x) - transmission.y * 0.29) * 32.0;
                pulse += exp(-wave * wave) * (1.0 - transmission.y / 1.2);
            }
            tint = mix(tint, vec3(1.0, 0.93, 0.74), min(1.0, pulse));
            light *= (0.76 + 1.8 * pulse) * smoothstep(0.0, 0.035, min(along, 1.0 - along));
        } else if (material == 3) {
            light *= 1.0 - smoothstep(0.27, 0.42, fract(along * 56.0));
        } else if (material == 4) {
            light *= smoothstep(0.0, 0.16, min(along, 1.0 - along));
        }
    }
    color = vec4(tint * light, min(light, 1.0));
}
