# version 330

struct Material {
    sampler2D diffuse_map;
    sampler2D bump_map;
    sampler2D normal_map;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
    bool enabled;
    bool use_diffuse_map;
    bool use_normal_map;
    bool use_bump_map;
    bool use_parallax_map;
};


in vec4 v_color;
in vec3 v_normal;
in vec2 v_texcoord;
in vec3 v_tangent;
in vec3 frag_pos;
in vec3 world_pos;
in vec3 light_pos;
in vec3 light_dir;
in vec4 frag_pos_light_space;

out vec4 frag_color;

uniform bool shading_enabled;
uniform Material mtl;
uniform vec3 light_color;
uniform bool is_directional;
uniform bool shadow_map_enabled;
uniform sampler2D shadow_map;

float calc_shadow(vec4 pos_in_light_space, vec3 normal, vec3 l) {
    float bias = max(0.0005 * (1.0 - dot(normal, l)), 0.0005);
    float shadow = 0.0;
    if (shadow_map_enabled) {
        vec3 proj_coords = pos_in_light_space.xyz / pos_in_light_space.w;
        proj_coords = proj_coords * 0.5 + 0.5;

        if (proj_coords.z > 1.0)
            return shadow;

        // Simplest percentage-closer filtering:
        //   sampling the surrounding texels of the depth map and average the results
        vec2 texel_size = 1.0 / textureSize(shadow_map, 0);

        float depth = proj_coords.z;
        for (int x = -1; x <= 1; ++x) {
            for (int y = -1; y <= 1; ++y) {
                float max_depth = texture(shadow_map, proj_coords.xy + vec2(x, y) * texel_size).r;
                shadow += depth - bias > max_depth ? 1.0 : 0.0;
            }
        }
        shadow /= 9.0;
    }

    return shadow;
}


// Blinn Phong BRDF in Camera Space
vec3 blinnPhongBRDF(vec3 light_dir, vec3 view_dir, vec3 normal, vec3 diffuse_color, vec3 specular_color, float shininess) {
    vec3 color = diffuse_color;
    vec3 half_dir = normalize(view_dir + light_dir);
    float spec_dot = max(dot(half_dir, normal), 0.0);
    color += pow(spec_dot, shininess) * specular_color;
    return color;
}

vec4 shading(vec3 ambient_color, vec3 light_dir, vec3 view_dir, vec3 light_color, vec3 n, vec3 diffuse_color, vec3 specular_color, float shininess, float shadow) {
    vec3 luminance = ambient_color.rgb * 0.2 + diffuse_color * 0.3;

    float illuminance = dot(light_dir, n);

    if (illuminance > 0.0) {
        vec3 brdf = blinnPhongBRDF(light_dir, view_dir, n, diffuse_color.rgb, specular_color.rgb, shininess);

        luminance += (1.0 - shadow) * brdf * illuminance * light_color.rgb * 0.6;
    }

    return vec4(luminance, 1.0);
}

mat3 tangentSpaceMatrix () {

    vec3 tangent = normalize(v_tangent);
    vec3 normal = normalize(v_normal);
    vec3 bitangent = normalize (cross(normal, tangent));

    return mat3(tangent, bitangent, normal);
}

vec2 parallaxMap (sampler2D tex) { 
    float parallax_factor = 10.0;

    vec3 view_dir = normalize(-frag_pos);

    float height =  texture(tex, v_texcoord).x;
    vec2 displacement = view_dir.xy / view_dir.z * (height * parallax_factor);

    return v_texcoord - displacement;
} 

vec3 bumpMap (sampler2D tex) {

    //https://developer.download.nvidia.com/CgTutorial/cg_tutorial_chapter08.html
    vec2 tex_size = 1.0/textureSize(tex, 0);


    // texture gradient (only x component is necessary since bump map is gray scale)
    vec2 grad = vec2(texture(tex, v_texcoord).x - texture(tex, v_texcoord+vec2(tex_size.x,0)).x,
                  texture(tex, v_texcoord+vec2(0,tex_size.y)).x - texture(tex, v_texcoord).x);

    // bump map multiplier (to enhance effect)
    grad *= 3.0;

    // cross product of vector (0, 1, grad.x) x (1, 0, grad.y)
    vec3 bump_map = normalize(vec3(-grad.y, grad.x, 1.0));

    // place bump_map in tangent space
    mat3 TBN = tangentSpaceMatrix ();
    bump_map = normalize(TBN * bump_map);

    return bump_map;

}

vec3 normalMap (sampler2D tex) {

    mat3 TBN = tangentSpaceMatrix ();

    vec3 normal_map = texture(tex, v_texcoord).xyz*2.0 - vec3(1.0);
    normal_map = normalize(TBN * normal_map);

    return normal_map;
}

void main() {
    vec3 _light_dir = vec3(0.0);
    if (is_directional) {
        _light_dir = -normalize(light_dir);
    } else {
        _light_dir = normalize(light_pos - frag_pos);
    }

    vec3 view_dir = normalize(-frag_pos);
    vec3 n = normalize(v_normal);

    vec4 diffuse_color;
    vec4 specular_color;
    float shininess;
    vec4 ambient_color;

    if (mtl.enabled) {
        if (mtl.use_diffuse_map) {
            diffuse_color = texture(mtl.diffuse_map, v_texcoord);
        } else {
            diffuse_color = vec4(mtl.diffuse, 1.0);
        }
    
        if (mtl.use_normal_map) {
            n = normalMap(mtl.normal_map);
        }

        else if (mtl.use_bump_map) {
            n = bumpMap(mtl.diffuse_map);
            diffuse_color = vec4(mtl.diffuse, 1.0);
        }
        else if (mtl.use_parallax_map) {
            vec2 tex_displacement = parallaxMap(mtl.diffuse_map);
            diffuse_color = texture(mtl.diffuse_map, tex_displacement);
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

    float shadow = calc_shadow(frag_pos_light_space, n, _light_dir);

    if (shading_enabled) {
        frag_color = shading(ambient_color.xyz, _light_dir, view_dir.xyz, light_color.xyz, n, diffuse_color.xyz, specular_color.xyz, shininess, shadow);
    } else {
        frag_color = diffuse_color;
    }

    //frag_color = texture(mtl.bump_map, v_texcoord);
}
