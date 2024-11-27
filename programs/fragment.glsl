#version 330 core

out vec4 color;

uniform vec2 u_resolution;
uniform sampler3D u_texture;

uniform vec3 u_cam_pos;
uniform vec3 u_cam_look_at;
uniform float u_yaw;
uniform float u_pitch;

const float PI = 3.14159265;
const float FOV = 0.7;
const float MAX_DISTANCE = 48.0;

struct RayReturn {
    vec3 intersection;
    ivec3 point;
    bool hit;
};

// Uses DDA https://en.wikipedia.org/wiki/Digital_differential_analyzer_(graphics_algorithm)
RayReturn ray_cast(vec3 ray_origin, vec3 ray_direction) {
    vec3 ray_step_size = vec3(
        sqrt(1.0 + pow(ray_direction.y / ray_direction.x, 2.0) + pow(ray_direction.z / ray_direction.x, 2.0)),
        sqrt(1.0 + pow(ray_direction.x / ray_direction.y, 2.0) + pow(ray_direction.z / ray_direction.y, 2.0)),
        sqrt(1.0 + pow(ray_direction.x / ray_direction.z, 2.0) + pow(ray_direction.y / ray_direction.z, 2.0))
    );
    ivec3 map_check = ivec3(int(ray_origin.x), int(ray_origin.y), int(ray_origin.z));

    vec3 ray_length, ray_step;
    if (ray_direction.x < 0.0) {
        ray_step.x = -1.0;
        ray_length.x = (ray_origin.x - float(map_check.x)) * ray_step_size.x;
    } else {
        ray_step.x = 1.0;
        ray_length.x = (float(map_check.x) + 1.0 - ray_origin.x) * ray_step_size.x;
    }

    if (ray_direction.y < 0.0) {
        ray_step.y = -1.0;
        ray_length.y = (ray_origin.y - float(map_check.y)) * ray_step_size.y;
    } else {
        ray_step.y = 1.0;
        ray_length.y = (float(map_check.y) + 1.0 - ray_origin.y) * ray_step_size.y;
    }

    if (ray_direction.z < 0.0) {
        ray_step.z = -1.0;
        ray_length.z = (ray_origin.z - float(map_check.z)) * ray_step_size.z;
    } else {
        ray_step.z = 1.0;
        ray_length.z = (float(map_check.z) + 1.0 - ray_origin.z) * ray_step_size.z;
    }

    ivec3 texture_size = textureSize(u_texture, 0);

    float distance = 0.0;
    bool tile_found = false;

    if (map_check.x >= 0 
            && map_check.x < texture_size.x
            && map_check.y >= 0
            && map_check.y < texture_size.y
            && map_check.z >= 0
            && map_check.z < texture_size.z) {
                vec4 texel = texelFetch(u_texture, map_check, 0);

                if (texel.a == 0.0) {
                    tile_found = true;
                }
            }

    while (!tile_found && distance < MAX_DISTANCE) {
        if (ray_length.x < ray_length.y && ray_length.x < ray_length.z) {
            map_check.x += int(ray_step.x);
            distance = ray_length.x;
            ray_length.x += ray_step_size.x;
        } else if (ray_length.y < ray_length.z) {
            map_check.y += int(ray_step.y);
            distance = ray_length.y;
            ray_length.y += ray_step_size.y;
        } else {
            map_check.z += int(ray_step.z);
            distance = ray_length.z;
            ray_length.z += ray_step_size.z;
        }

        if (map_check.x >= 0 
            && map_check.x < texture_size.x
            && map_check.y >= 0
            && map_check.y < texture_size.y
            && map_check.z >= 0
            && map_check.z < texture_size.z) {
                vec4 texel = texelFetch(u_texture, map_check, 0);

                if (texel.a == 0.0) {
                    tile_found = true;
                }
            }
    }

    vec3 intersection = ray_origin + ray_direction * distance;
    intersection = round(intersection * 1000.0) / 1000.0;
    if (tile_found) {
        return RayReturn(intersection, map_check, true);
    }
    return RayReturn(intersection, map_check, false);
}

mat3 get_cam(vec3 ro) {
    vec3 cam_f = normalize(vec3(u_cam_look_at - ro));
    vec3 cam_r = normalize(cross(vec3(0, 1, 0), cam_f));
    vec3 cam_u = cross(cam_f, cam_r);
    return mat3(cam_r, cam_u, cam_f);
}

vec3 get_normal(ivec3 point, vec3 intersection) {
    vec3 local = intersection - vec3(point);
    float epsilon = 0.0001;

    if (abs(local.x - 1.0) < epsilon || abs(local.x + 1.0) < epsilon) {
        return vec3(sign(local.x), 0.0, 0.0);
    } else if (abs(local.y - 1.0) < epsilon || abs(local.y + 1.0) < epsilon) {
        return vec3(0.0, sign(local.y), 0.0);
    } else if (abs(local.z - 1.0) < epsilon || abs(local.z + 1.0) < epsilon) {
        return vec3(0.0, 0.0, sign(local.z));
    }

    return vec3(0.0, 0.0, 0.0);
}


vec3 get_light(ivec3 point, vec3 intersection, vec3 ray_direction, vec3 color) {
    vec3 texture_size = textureSize(u_texture, 0);
    vec3 light_pos = vec3(texture_size.x / 2.0, texture_size.y + 100.0, texture_size.z / 2.0);
    vec3 L = normalize(light_pos - intersection);
    vec3 N = get_normal(point, intersection);

    vec3 diffuse = color * clamp(dot(L, N), 0.05, 1.0);

    RayReturn shadow = ray_cast(intersection + N * 0.1, normalize(light_pos - intersection));
    if (length(shadow.intersection - intersection) < length(light_pos - intersection) && shadow.hit) return diffuse * 0.33;
    return diffuse;
}


void render(inout vec3 col, in vec2 uv) {
    vec3 ray_origin = u_cam_pos;
    vec3 ray_direction = get_cam(ray_origin) * normalize(vec3(uv, FOV));

    RayReturn casted = ray_cast(ray_origin, ray_direction);
    if (casted.hit) {
        vec2 delta = casted.intersection.xz - ray_origin.xz;
        float distance = length(delta);

        col = texelFetch(u_texture, casted.point, 0).rgb;

        col = get_light(casted.point, casted.intersection, ray_direction, col);

        float fog_factor = (MAX_DISTANCE - distance) / MAX_DISTANCE;
        col = mix(vec3(0.1, 0.1, 0.1), col, fog_factor);
    }
}

void main() {
    vec2 uv = (2.0 * gl_FragCoord.xy - u_resolution.xy) / u_resolution.y;

    vec3 col = vec3(0.1, 0.1, 0.1);
    render(col, uv);

    col = pow(col, vec3(0.4545));
    color = vec4(col, 1.0);
}