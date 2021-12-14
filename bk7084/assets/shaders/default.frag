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
in vec3 v_tangent;
in vec3 frag_pos;
in vec3 world_pos;
in vec3 light_pos;

out vec4 frag_color;

uniform bool shading_enabled;
uniform Material mtl;

bool use_normal_map = false;
bool use_bump_map = false;


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

vec3 bumpMap (sampler2D tex) {

    //https://developer.download.nvidia.com/CgTutorial/cg_tutorial_chapter08.html

    float tex_size = 1.0/1024.0;

    vec3 tangent = normalize(v_tangent);
    vec3 normal = normalize(v_normal);
    vec3 bitangent = normalize (cross(normal, tangent));

    mat3 TBN = mat3(tangent, bitangent, normal);

    // texture gradient (only x component is necessary since bump map is gray scale)
    vec2 grad = vec2(texture(tex, v_texcoord).x - texture(tex, v_texcoord+vec2(tex_size,0)).x,
                  texture(tex, v_texcoord+vec2(0,tex_size)).x - texture(tex, v_texcoord).x);

    // bump map multiplier (to enhance effect)
    grad *= 10.0;

    // cross product of vector (0, 1, grad.x) x (1, 0, grad.y)
    vec3 bump_map = normalize(vec3(-grad.y, grad.x, 1.0));

    // place bump_map in tangent space
    bump_map = normalize(TBN * bump_map);

    return bump_map;

}

vec3 normalMap (sampler2D tex) {

    vec3 tangent = normalize(v_tangent);
    vec3 normal = normalize(v_normal);
    vec3 bitangent = normalize (cross(normal, tangent));

    mat3 TBN = mat3(tangent, bitangent, normal);

    vec3 normal_map = texture(tex, v_texcoord).xyz*2.0 - vec3(1.0);
    normal_map = normalize(TBN * normal_map);

    return normal_map;
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
    
        if (use_normal_map) {
            n = normalMap(mtl.diffuse_map);
            diffuse_color = vec4(mtl.diffuse, 1.0);
        }    
        else if (use_bump_map) {
            n = bumpMap(mtl.diffuse_map);
            diffuse_color = vec4(mtl.diffuse, 1.0);            
        }    
        else if (mtl.use_diffuse_map) {
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
        shininess = 1.0;
        ambient_color = vec4(0.5, 0.5, 0.5, 1.0);
    }


    if (shading_enabled) {
        frag_color = shading(ambient_color.xyz, light_dir.xyz, view_dir.xyz, light_color.xyz, n, diffuse_color.xyz, specular_color.xyz, shininess);
    } else {
        frag_color = diffuse_color;
    }

    //frag_color = vec4(normalize(n), 1.0);
}
