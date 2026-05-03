#version 330

in vec2 fragTexCoord;
out vec4 finalColor;

uniform sampler2D texture0;
uniform vec2 resolution;

void main() {
    vec2 uv = fragTexCoord;
    vec2 texel = 1.0 / resolution;
    vec4 col = texture(texture0, uv);
    vec4 bloom = vec4(0.0);
    float total = 0.0;

    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            float r2 = float(x*x + y*y);
            float gauss = exp(-r2 / 2.0);
            vec2 offset = vec2(float(x), float(y)) * texel * 3.0;
            vec4 s = texture(texture0, uv + offset);
            float lum = dot(s.rgb, vec3(0.2126, 0.7152, 0.0722));
            // seuil à 0.55 : seules les étoiles vraiment lumineuses déclenchent le bloom
            float weight = max(0.0, lum - 0.55) * gauss;
            bloom += s * weight;
            total += weight;
        }
    }

    if (total > 0.0) bloom /= total;
    finalColor = col + bloom * 0.8;
    finalColor.a = 1.0;
}
