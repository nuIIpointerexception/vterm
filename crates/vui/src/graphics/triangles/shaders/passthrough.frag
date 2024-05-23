#version 450
#extension GL_ARB_separate_shader_objects: enable
#extension GL_EXT_nonuniform_qualifier : require

layout(location = 0) in vec4 vertex_color;
layout(location = 1) in vec2 uv;
layout(location = 2) flat in int texIndex;

layout(location = 0) out vec4 frag_color;

layout(binding = 2) uniform sampler2D textures[];

void main() {
    vec4 tex_color = texture(textures[nonuniformEXT(texIndex)], uv);
    frag_color = tex_color * vertex_color;
}
