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
in vec3 light_pos;
in vec3 frag_pos;
in vec3 world_pos;
in vec2 v_texcoord;

out vec4 frag_color;

uniform bool do_shading;
uniform Material mtl;

vec4 shading(vec3 diffuse_color) {
    vec3 light_color = vec3(1.0, 1.0, 1.0);

    // face normal approximation
    vec3 x = dFdx(world_pos);
    vec3 y = dFdy(world_pos);
    vec3 face_normal = cross(x, y);

    // diffuse
    vec3 light_dir = normalize(light_pos - frag_pos);
    float diff = max(dot(normalize(face_normal), light_dir), 0.0);

    vec3 mtl_diffuse;

    if (mtl.use_diffuse_map) {
        mtl_diffuse = texture(mtl.diffuse_map, v_texcoord).xyz;
    } else {
        mtl_diffuse = mtl.diffuse.xyz;
    }

    vec3 diffuse;

    if (!mtl.enabled) {
        diffuse = 0.4 * (diff * light_color * v_color.xyz) + v_color.xyz * 0.5;
    } else {
        diffuse = 0.4 * (diff * light_color * mtl_diffuse) + mtl.ambient.xyz * 0.5;
    }

    //return vec4(diffuse, 1.0);
    return vec4(mtl_diffuse, 1.0);
}

void main() {
    vec4 diffuse_color;

    if (mtl.enabled) {
         if (mtl.use_diffuse_map) {
             diffuse_color = texture(mtl.diffuse_map, v_texcoord);
         } else {
             diffuse_color = vec4(mtl.diffuse, 1.0);
         }
    } else{
        diffuse_color = v_color;
    }

    if (do_shading) {
        frag_color = shading(diffuse_color.xyz);
    } else {
        frag_color = diffuse_color;
    }
}
