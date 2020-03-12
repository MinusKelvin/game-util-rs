#version 330 core

in vec3 texcoord;
in vec4 col;

uniform sampler2DArray tex;

out vec4 color;

void main() {
    color = texture(tex, texcoord) * col;
}