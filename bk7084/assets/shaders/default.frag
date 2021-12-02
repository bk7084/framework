# version 330

struct Material {
    sampler2D diffuse_map;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
    bool enabled;
    bool use_diffuse_map;
};


in vec4 v_color;
in vec3 v_normal;
in vec2 v_texcoord;
in vec3 frag_pos;
in vec3 world_pos;
in vec3 light_pos;

out vec4 frag_color;

uniform bool shading_enabled;
uniform Material mtl;


// Blinn Phong BRDF in Camera Space
vec3 blinnPhongBRDF(vec3 light_dir, vec3 view_dir, vec3 normal, vec3 diffuse_color, vec3 specular_color, float shininess) {
    vec3 color = diffuse_color;
    vec3 half_dir = normalize(view_dir + light_dir);
    float spec_dot = max(dot(half_dir, normal), 0.0);
    color += pow(spec_dot, shininess) * specular_color;
    return color;
}

vec4 shading(vec3 ambient_color, vec3 light_dir, vec3 view_dir, vec3 light_color, vec3 n, vec3 diffuse_color, vec3 specular_color, float shininess) {
    vec3 luminance = ambient_color.rgb * 0.1 + diffuse_color * 0.3;

    float illuminance = dot(light_dir, n);
    if (illuminance > 0.0) {
        vec3 brdf = blinnPhongBRDF(light_dir, view_dir, n, diffuse_color.rgb, specular_color.rgb, shininess);

        luminance += brdf * illuminance * light_color.rgb * 0.6;
    }

    return vec4(luminance, 1.0);
}

void main() {
    vec3 light_dir = normalize(light_pos - frag_pos);
    vec3 view_dir = normalize(-frag_pos);
    vec3 n = normalize(v_normal);

    vec4 diffuse_color;
    vec4 specular_color;
    float shininess;
    vec4 ambient_color;
    vec4 light_color = vec4(0.8, 0.8, 0.8, 1.0);

    if (mtl.enabled) {
         if (mtl.use_diffuse_map) {
             diffuse_color = texture(mtl.diffuse_map, v_texcoord);
         } else {
             diffuse_color = vec4(mtl.diffuse, 1.0);
         }

         specular_color = vec4(mtl.specular, 1.0);
         shininess = mtl.shininess;
         ambient_color = vec4(mtl.ambient, 1.0);
    } else {
        diffuse_color = v_color;
        specular_color = vec4(1.0, 1.0, 1.0, 1.0);
        shininess = 0;
        ambient_color = vec4(0.5, 0.5, 0.5, 1.0);
    }


    if (shading_enabled) {
        frag_color = shading(ambient_color.xyz, light_dir.xyz, view_dir.xyz, light_color.xyz, n, diffuse_color.xyz, specular_color.xyz, shininess);
    } else {
        frag_color = diffuse_color;
    }
}
