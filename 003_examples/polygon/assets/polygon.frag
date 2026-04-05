#version 450

layout(set = 0, binding = 0) uniform UniformBufferObject {
    vec4 CameraEye;
    vec4 FogColor;
} ubo;

layout(location = 0) in vec4 mColor;
layout(location = 1) in vec4 mVertex;

layout(location = 0) out vec4 outColor;

float getFogFactor(float d)
{
    const float FogMax = 20.0;
    const float FogMin = 10.0;

    if (d>=FogMax) return 1.0;
    if (d<=FogMin) return 0.0;

    return 1.0 - (FogMax - d) / (FogMax - FogMin);
}

void main(void)
{
    vec4 V = mVertex;
    float d = distance(ubo.CameraEye, V);
    float alpha = getFogFactor(d);

    outColor = mix(mColor, ubo.FogColor, alpha);
}
