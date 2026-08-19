#version 450 core

// Define vertices for a triangle
const vec2 positions[3] = vec2[3](
    vec2(0.0, 0.5),   // Top
    vec2(-0.5, -0.5), // Bottom-left
    vec2(0.5, -0.5)   // Bottom-right
);

void main() {
    // Set the position of the vertex using the index
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
