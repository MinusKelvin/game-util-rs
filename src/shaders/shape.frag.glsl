#ifdef GL_ES
precision lowp float;
#endif

in vec4 col;

out vec4 color;

void main() {
    color = col;
}