#version 330 core

in VS_OUT {
    vec3 color;
} IN;

out vec4 frag_color;

uniform float red;

void main() {
    frag_color = vec4(red, IN.color.yz, 1.0);
}
