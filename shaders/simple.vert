#version 450

layout(set = 0, binding = 0) uniform Translation { mat4 matrix; } translation;
layout(set = 0, binding = 1) uniform Scale { mat4 matrix; } scale;
layout(set = 0, binding = 2) uniform Rotation { mat4 matrix; } rotation;
layout(set = 1, binding = 0) uniform View { mat4 matrix; } view;
layout(set = 2, binding = 0) uniform Projection { mat4 matrix; } projection;


layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inTangent;
layout(location = 3) in vec2 inUV;

layout(location = 0) out vec3 outNormal;
layout(location = 1) out vec2 outUV;
layout(location = 2) out vec3 outPosition;

void main() {
  outNormal = (rotation.matrix * vec4(inNormal, 1.0)).xyz;
  outPosition = 
	  (translation.matrix * 
	  scale.matrix *
	  rotation.matrix *
	  vec4(inPosition, 1.0)).xyz;
  outUV = inUV;
  gl_Position = 
	  projection.matrix * 
	  view.matrix * 
	  translation.matrix * 
	  scale.matrix *
	  rotation.matrix *
	  vec4(inPosition, 1.0);
}
