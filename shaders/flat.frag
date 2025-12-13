#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec3 inNormal;

layout(set = 3, binding = 0) uniform Light { vec3 dir; } light;

void main() {
    outColor = vec4(vec3(dot(-normalize(light.dir), inNormal) - 0.2), 1.0);
}
