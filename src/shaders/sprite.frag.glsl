in vec3 texcoord;
in vec4 col;

uniform sampler2DArray sprites;

out vec4 color;

void main() {
    color = texture(sprites, texcoord) * col;
}