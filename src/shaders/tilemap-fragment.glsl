#version 330 core

in vec2 coord;

uniform usampler2D tilemap;
uniform sampler2DArray tileset;

out vec4 color;

void main() {
    vec2 tilespace = coord - floor(coord);

    uint tile = texelFetch(tilemap, ivec2(coord), 0).r;

    color = textureGrad(tileset, vec3(tilespace, float(tile)), dFdx(coord), dFdy(coord));
}
