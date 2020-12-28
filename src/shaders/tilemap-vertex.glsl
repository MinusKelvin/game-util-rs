#version 300 es

uniform vec2 size;
uniform vec2 offset;
uniform mat4 proj;

out vec2 coord;

void main() {
    const vec2 COORDS[4] = vec2[4](
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(1.0, 1.0)
    );

	vec2 pos = size * COORDS[gl_VertexID];
    gl_Position = proj * vec4(pos, 0.0, 1.0);
    coord = offset + pos;
}
