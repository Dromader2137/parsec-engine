#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec3 inNormal;

void main() {
	vec3 lightDir = normalize(vec3(1.0, -1.0, 1.0));
    outColor = vec4(vec3(dot(-lightDir, inNormal) - 0.2), 1.0);
}
