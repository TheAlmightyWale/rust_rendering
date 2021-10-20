use crate::lights::Light;
use crate::objects::Sphere;
use crate::properties::Color;
use crate::properties::Material;
use crate::properties::BG_COLOR;
use crate::scene::Scene;
use crate::state::Surface;
use cgmath::InnerSpace; //Dot product and magnitude

static MIN_Z: f32 = 1.0;
static REFLECTION_RECURSION_LIMIT: u32 = 3;

pub fn ray_trace(scene: &Scene, surface: &mut dyn Surface) {
    //Get bounds of drawing sruface
    let viewport_width = surface.get_width() as f32;
    let viewport_height = surface.get_height() as f32;
    let size = cgmath::Vector2::<f32> {
        x: viewport_width,
        y: viewport_height,
    };
    let origin = cgmath::Vector3::new(0.0, 0.0, 0.0);
    for y in 0..surface.get_height() {
        for x in 0..surface.get_width() {
            //Centering x and y gives us a camera view centered at 0,0,0, rather than having the far left of the view starting at 0,0,0
            let centered_x = x as f32 - (viewport_width / 2.0);
            let centered_y = y as f32 - (viewport_height / 2.0);
            let direction = canvas_to_viewport(centered_x, centered_y, size);
            let color = trace_ray(
                &origin,
                &direction,
                1.0,
                f32::INFINITY,
                scene,
                REFLECTION_RECURSION_LIMIT,
            );
            surface.set_pixel(x, y, &color);
        }
    }
}

fn canvas_to_viewport(x: f32, y: f32, size: cgmath::Vector2<f32>) -> cgmath::Vector3<f32> {
    cgmath::Vector3::<f32> {
        x: (x / size.x),
        y: (y / size.y),
        z: MIN_Z,
    }
}

//min and max distance are measured as the parameter t in the vector equation P = Q + t(V - Q), where V and Q are 2 points
fn trace_ray(
    origin: &cgmath::Vector3<f32>,
    ray_direction: &cgmath::Vector3<f32>,
    min_distance: f32,
    max_distance: f32,
    scene: &Scene,
    reflection_recursion_depth: u32,
) -> Color<u8> {
    let (closest_sphere, closest_t) =
        closest_intersection(origin, ray_direction, scene, min_distance, max_distance);
    match closest_sphere {
        Some(sphere) => {
            let intersection = origin + closest_t * ray_direction;
            let normal = (intersection - sphere.center).normalize();
            let mut local_color = sphere.get_color()
                * compute_lighting(
                    scene,
                    &intersection,
                    &normal,
                    &sphere.material,
                    &(ray_direction * -1.0),
                );
            if reflection_recursion_depth > 0 {
                if let Material::Specular { reflectiveness, .. } = sphere.material {
                    //Compute reflected colors
                    let reversed_ray = ray_direction * -1.0;
                    let reflected_ray = reflect_ray(&normal, &reversed_ray);
                    let reflected_color = trace_ray(
                        &intersection,
                        &reflected_ray,
                        0.0001,
                        f32::INFINITY,
                        scene,
                        reflection_recursion_depth - 1,
                    );
                    local_color =
                        local_color * (1.0 - reflectiveness) + reflected_color * reflectiveness;
                }
            }
            local_color
        }
        None => BG_COLOR,
    }
}

fn closest_intersection<'scene_lifetime>(
    origin: &cgmath::Vector3<f32>,
    direction: &cgmath::Vector3<f32>,
    scene: &'scene_lifetime Scene,
    min_distance: f32,
    max_distance: f32,
) -> (Option<&'scene_lifetime Sphere>, f32) {
    let mut closest_t = f32::INFINITY;
    let mut closest_sphere: Option<&Sphere> = None;
    for sphere in scene.objects.iter() {
        let determinants = intersect_ray_sphere(origin, direction, sphere);
        if (min_distance..max_distance).contains(&determinants.0) && determinants.0 < closest_t {
            closest_t = determinants.0;
            closest_sphere = Some(&sphere);
        }

        if (min_distance..max_distance).contains(&determinants.1) && determinants.1 < closest_t {
            closest_t = determinants.1;
            closest_sphere = Some(&sphere);
        }
    }
    (closest_sphere, closest_t)
}

//returns the determinants of the quadratic equation, f32::INFINITY(no intersection), both determinants equal (tangent), two solutions (intersection)
fn intersect_ray_sphere(
    origin: &cgmath::Vector3<f32>,
    ray_direction: &cgmath::Vector3<f32>,
    sphere: &Sphere,
) -> (f32, f32) {
    let radius = sphere.radius;
    let origin_sphere = origin - sphere.center;
    //Quadratic equation
    let a = cgmath::dot(*ray_direction, *ray_direction);
    let b = 2.0 * cgmath::dot(origin_sphere, *ray_direction);
    let c = cgmath::dot(origin_sphere, origin_sphere) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        (f32::INFINITY, f32::INFINITY)
    } else {
        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminant.sqrt()) / (2.0 * a);
        (t1, t2)
    }
}

fn compute_lighting(
    scene: &Scene,
    intersection_point: &cgmath::Vector3<f32>,
    surface_normal: &cgmath::Vector3<f32>,
    material: &Material,
    view: &cgmath::Vector3<f32>,
) -> Color<f32> {
    let mut total_intensity = Color::<f32> {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    for light in scene.lights.iter() {
        total_intensity = total_intensity
            + calculate_light_intensity(
                light,
                intersection_point,
                surface_normal,
                material,
                view,
                scene,
            );
    }
    total_intensity
}
fn calculate_light_intensity(
    light: &Light,
    intersection_point: &cgmath::Vector3<f32>,
    surface_normal: &cgmath::Vector3<f32>,
    material: &Material,
    view: &cgmath::Vector3<f32>,
    scene: &Scene,
) -> Color<f32> {
    let mut light_intensity = Color::<f32> {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    //Calculate light
    match light {
        Light::Directional {
            direction,
            intensity,
        } => {
            if !point_in_shadow(intersection_point, direction, f32::INFINITY, scene) {
                light_intensity = calculate_directional_light(
                    direction,
                    intensity,
                    surface_normal,
                    material,
                    view,
                )
            }
        }
        Light::Point {
            position,
            intensity,
        } => {
            let direction = position - intersection_point;
            if !point_in_shadow(intersection_point, &direction, 1.0, scene) {
                light_intensity = calculate_directional_light(
                    &direction,
                    intensity,
                    surface_normal,
                    material,
                    view,
                )
            }
        }
        Light::Ambient { intensity } => light_intensity = *intensity,
    }
    light_intensity
}

fn point_in_shadow(
    intersection_point: &cgmath::Vector3<f32>,
    direction: &cgmath::Vector3<f32>,
    t_max: f32,
    scene: &Scene,
) -> bool {
    //Shadow check
    let (shadow_sphere, _shadow_t) =
        closest_intersection(intersection_point, direction, scene, 0.0001, t_max);
    match shadow_sphere {
        Some(_sphere) => true,
        None => false,
    }
}

fn calculate_directional_light(
    direction: &cgmath::Vector3<f32>,
    intensity: &Color<f32>,
    surface_normal: &cgmath::Vector3<f32>,
    material: &Material,
    view: &cgmath::Vector3<f32>,
) -> Color<f32> {
    let mut light_color_to_add = Color::<f32> {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    //Diffuse
    let dot_normal_direction = cgmath::dot(*surface_normal, *direction);
    if dot_normal_direction > 0.0 {
        let scale = dot_normal_direction / (surface_normal.magnitude() * direction.magnitude());
        light_color_to_add = *intensity * scale;
    }
    //Specular
    if let Material::Specular {
        specular, color, ..
    } = material
    {
        let reflection = reflect_ray(surface_normal, direction);
        let reflection_dot_view = cgmath::dot(reflection, *view);
        if reflection_dot_view > 0.0 {
            let specular_scale: f32 =
                reflection_dot_view / (reflection.magnitude() * view.magnitude());
            light_color_to_add = light_color_to_add + *intensity * specular_scale.powf(*specular);
            light_color_to_add.a = color.a as f32 / u8::MAX as f32;
        }
    }
    light_color_to_add
}

fn reflect_ray(
    surface_normal: &cgmath::Vector3<f32>,
    ray: &cgmath::Vector3<f32>,
) -> cgmath::Vector3<f32> {
    2.0 * surface_normal * cgmath::dot(*surface_normal, *ray) - ray
}
