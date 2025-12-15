#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec3 inNormal;
layout(location = 1) in vec2 inUV;

layout(set = 3, binding = 0) uniform Light { vec3 dir; } light;
layout(set = 4, binding = 0) uniform sampler2D tex;

void main() {
    float intensity = abs(dot(-normalize(light.dir), inNormal)) + 0.1;
	vec3 color = texture(tex, inUV).rgb;
	outColor = vec4(color * intensity, 1.0);
}
