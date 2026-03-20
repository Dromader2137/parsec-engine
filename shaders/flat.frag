#version 450

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec3 inNormal;
layout(location = 1) in vec2 inUV;
layout(location = 2) in vec3 inPosition;
layout(location = 3) noperspective in vec4 inLPosition;

layout(set = 3, binding = 0) uniform Light { vec3 dir; mat4 mat; } light;
layout(set = 4, binding = 0) uniform sampler2D tex;
layout(set = 5, binding = 0) uniform sampler2D shadow;

void main() {
	float light_angle = dot(-normalize(light.dir), inNormal);
    float intensity = clamp(light_angle, 0.05, 1.0);
	vec3 light_pos = inLPosition.xyz / inLPosition.w;
	float cam_depth = texture(shadow, ((light_pos.xy + 1.0) / 2.0)).r;
	if (light_pos.z - 0.0005 >= cam_depth) {
		intensity = 0.05;
	}
	vec3 color = texture(tex, inUV).rgb;
	outColor = vec4(color * intensity, 1.0);
}
