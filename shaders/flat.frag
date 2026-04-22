#version 450

struct DirLightData {
    mat4 world_to_light;
	vec4 atlas_clip;
    vec3 direction;
    vec3 color;
};

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec3 inNormal;
layout(location = 1) in vec2 inUV;
layout(location = 2) in vec3 inPosition;

layout(std140, set = 3, binding = 0) readonly buffer LightData {
    uint light_count;
    DirLightData data[32];
} lightData;

layout(set = 4, binding = 0) uniform sampler2D tex;
layout(set = 5, binding = 0) uniform sampler2D shadow;

int num_rings = 2;

void main() {
    float light_angle = dot(-normalize(lightData.data[0].direction), inNormal);
    float intensity = clamp(light_angle, 0.02, 1.0);
    vec3 light_pos = (lightData.data[0].world_to_light * vec4(inPosition, 1.0)).xyz;
	vec2 lo = lightData.data[0].atlas_clip.xy * 2.0 - 1.0;
	vec2 hi = lightData.data[0].atlas_clip.zw * 2.0 - 1.0;
	light_pos.xy = lo + (light_pos.xy + 1.0) * (hi - lo) * 0.5;
    float shadow_samples = 0.0;
    int total_points = 0;
    for (int ring = 1; ring <= num_rings; ++ring) {
        int point_count = ring * ring;
        total_points += point_count;
        for (int i = 0; i < point_count; ++i) {
            float theta = i / float(point_count) * 6.283;
            vec2 ofst = vec2(sin(theta), cos(theta)) * float(ring) * 0.0003;
            float cam_depth = texture(shadow, ((light_pos.xy + ofst + 1.0) / 2.0)).r;
            if (light_pos.z - 0.001 <= cam_depth) {
                shadow_samples += 1.0;
            }
        }
    }
    intensity *= shadow_samples / float(total_points);
    intensity = clamp(intensity, 0.02, 1.0);
    vec3 color = texture(tex, inUV).rgb * lightData.data[0].color;
    outColor = vec4(color * intensity, 1.0);
}
