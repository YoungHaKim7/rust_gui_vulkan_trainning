#version 450 core

// Output color for the fragment
layout(location=0) out vec4 f_color;

void main() {
    // Set fragment color (RGBA)
    f_color = vec4(0.3, 0.2, 0.1, 1.0);
}
