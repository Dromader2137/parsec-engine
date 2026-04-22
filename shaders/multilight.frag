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

layout(set = 4, binding = 0) uniform sampler2D albedo;
layout(set = 5, binding = 0) uniform sampler2D shadow;

void main() {
    vec3 finalColor = vec3(0.0);
    for (int lightIdx = 0; lightIdx < lightData.light_count; ++lightIdx) {
        float light_angle = dot(-normalize(lightData.data[lightIdx].direction), inNormal);
        float light_intensity = clamp(light_angle, 0.02, 1.0);
        vec3 light_space_position = (lightData.data[lightIdx].world_to_light * vec4(inPosition, 1.0)).xyz;
        if (light_space_position.x > 1.0 && light_space_position.y > 1.0 && light_space_position.x < -1.0 && light_space_position.y < -1.0) {
            finalColor += texture(albedo, inUV).rgb * lightData.data[lightIdx].color * light_intensity;
            continue;
        }
        vec2 lo = lightData.data[lightIdx].atlas_clip.xy * 2.0 - 1.0;
        vec2 hi = lightData.data[lightIdx].atlas_clip.zw * 2.0 - 1.0;
        light_space_position.xy = lo + (light_space_position.xy + 1.0) * (hi - lo) * 0.5;
        float fragement_depth = texture(shadow, light_space_position.xy / 2.0 + 0.5).r;
        if (light_space_position.z - 0.002 <= fragement_depth) {
            finalColor += texture(albedo, inUV).rgb * lightData.data[lightIdx].color * light_intensity;
        }
    }
    outColor = vec4(finalColor, 1.0);
}
