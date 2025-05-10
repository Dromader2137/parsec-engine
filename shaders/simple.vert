#version 450

layout(binding = 0) uniform MVPBuffer {
  mat4 matrix;
} mvp;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 outColor;

void main() {
    outColor = inColor;
    gl_Position = mvp.matrix * vec4(inPosition, 0.0, 1.0);
}
