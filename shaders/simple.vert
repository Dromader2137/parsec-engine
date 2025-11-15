#version 450

layout(set = 0, binding = 0) uniform Model { mat4 matrix; } model;
layout(set = 1, binding = 0) uniform View { mat4 matrix; } view;
layout(set = 2, binding = 0) uniform Projection { mat4 matrix; } projection;


layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inTangent;
layout(location = 3) in vec2 inUV;

layout(location = 0) out vec3 outNormal;

void main() {
  outNormal = inNormal;
  gl_Position = projection.matrix * view.matrix * model.matrix * vec4(inPosition, 1.0);
}
