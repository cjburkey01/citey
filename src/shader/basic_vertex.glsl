#version 330 core

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_color;

out VS_OUT {
    vec3 color;
} OUT;

uniform mat4 projection_matrix;

void main() {
    OUT.color = vertex_color;

    gl_Position = projection_matrix
                * vec4(vertex_position, 1.0);
}
