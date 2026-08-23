#version 450

// Push constants are how we hand tiny per-frame data to the GPU without any buffers.
layout(push_constant) uniform Params {
    // x = framebuffer width, y = framebuffer height, z = circle radius (all in pixels)
    vec4 data;
} pc;

// One corner of the quad that will contain the circle, as two triangles.
const vec2 CORNERS[6] = vec2[6](
    vec2(-1.0, -1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2(-1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2( 1.0,  1.0)
);

// Position relative to the circle center, in pixels; interpolated for the fragment shader.
layout(location = 0) out vec2 v_local;

void main() {
    vec2 resolution = max(pc.data.xy, vec2(1.0));
    float radius = pc.data.z;

    // Make the quad slightly larger than the circle so the antialiased rim fits inside it.
    float half_size = radius + max(2.0, radius * 0.01);

    vec2 local_px = CORNERS[gl_VertexIndex] * half_size;

    // Pixel space -> clip space. No vertex buffer needed: the quad is generated here.
    gl_Position = vec4(local_px * 2.0 / resolution, 0.0, 1.0);

    v_local = local_px;
}
