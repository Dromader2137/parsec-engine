#version 450

struct DirLightData {
    mat4 world_to_light;
	vec4 atlas_clip;
    vec3 direction;
    vec3 color;
};

layout(set = 0, binding = 0) uniform Translation {
    mat4 matrix;
} translation;
layout(set = 0, binding = 1) uniform Scale {
    mat4 matrix;
} scale;
layout(set = 0, binding = 2) uniform Rotation {
    mat4 matrix;
} rotation;
layout(std140, set = 1, binding = 0) readonly buffer LightData {
    uint light_count;
    DirLightData data[32];
} lightData;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inTangent;
layout(location = 3) in vec2 inUV;

out gl_PerVertex {
	vec4 gl_Position;
	float gl_ClipDistance[4];
};

void main() {
    vec4 pos =
        lightData.data[gl_InstanceIndex].world_to_light *
            translation.matrix *
            scale.matrix *
            rotation.matrix *
            vec4(inPosition, 1.0);
	vec2 lo = lightData.data[gl_InstanceIndex].atlas_clip.xy * 2.0 - 1.0;
	vec2 hi = lightData.data[gl_InstanceIndex].atlas_clip.zw * 2.0 - 1.0;
	pos.xy = lo + (pos.xy + 1.0) * (hi - lo) * 0.5;
    gl_ClipDistance[0] = pos.x - lo.x * pos.w;
    gl_ClipDistance[1] = hi.x * pos.w - pos.x;
    gl_ClipDistance[2] = pos.y - lo.y * pos.w;
    gl_ClipDistance[3] = hi.y * pos.w - pos.y;
	gl_Position = pos;
}
