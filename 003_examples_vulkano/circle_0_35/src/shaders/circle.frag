#version 450

// Interpolated pixel offset from the circle center, produced by the vertex shader.
layout(location = 0) in vec2 v_local;

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform Params {
    // x = framebuffer width, y = framebuffer height, z = circle radius (all in pixels)
    vec4 data;
} pc;

void main() {
    float radius = pc.data.z;

    // Signed distance from the center, normalized to "radii".
    float d = length(v_local) / radius;

    // Feather the edge over roughly two pixels for antialiasing.
    float aa_px = max(2.0, radius * 0.005) / radius;
    float alpha = 1.0 - smoothstep(1.0 - aa_px, 1.0, d);

    // Nothing to write outside the circle.
    if (alpha <= 0.0) {
        discard;
    }

    f_color = vec4(1.0, 0.45, 0.15, alpha);
}
