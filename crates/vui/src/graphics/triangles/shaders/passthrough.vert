#version 450
#extension GL_ARB_separate_shader_objects: enable

struct Vertex
{
    vec4 pos;
    vec4 rgba;
    vec2 uv;
    int texIndex;
};

layout(std140, set=0, binding=0) readonly buffer SBO { Vertex data[]; } sbo;
layout(set=0, binding=1) readonly uniform UniformBufferObject {
    mat4 view_projection;
} ubo;

layout(location = 0) out vec4 vertex_color;
layout(location = 1) out vec2 uv;
layout(location = 2) flat out int texIndex;

void main() {
    Vertex vert = sbo.data[gl_VertexIndex];
    vertex_color = vert.rgba;
    uv = vert.uv;
    texIndex = vert.texIndex;
    gl_Position = ubo.view_projection * vert.pos;
}
