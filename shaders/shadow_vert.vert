#version 450

struct DirLightData {
    mat4 world_to_light;
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

void main() {
    gl_Position =
        lightData.data[gl_InstanceIndex].world_to_light *
            translation.matrix *
            scale.matrix *
            rotation.matrix *
            vec4(inPosition, 1.0);
    ;
}
