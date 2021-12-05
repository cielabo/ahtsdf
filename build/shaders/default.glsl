layout(std430, binding = 0) buffer normals {
    vec3 normal[];
};
layout(std430, binding = 1) buffer centers {
	vec3 center[];
};
layout(std430, binding = 2) buffer rad_vec {
	vec3 radius_vec[];
};
layout(std430, binding = 3) buffer angles {
	float angle[];
};
uniform uint num_segments;
uniform vec3 planes;

uniform float u_time;
uniform vec2 u_resolution;
uniform vec4 u_mouse;

out vec4 fragColor;

#define TORUS_THICKNESS 0.1

#define MAX_STEPS 100
#define MAX_DIST 100.
#define HIT_DIST 1e-4
#define PI 3.1415926535

// ======================================= UTILITY FUNCTIONS ==================================//

vec3 rot(vec3 p, vec3 ax, float an){
  return mix(dot(ax, p)*ax, p, cos(an)) + cross(ax,p)*sin(an);
}

vec3 hsv2rgb(vec3 c){
	vec3 conv = vec3(1, 2.0/3.0, 1.0/3.0);
	vec3 col = clamp(abs(6*fract(vec3(c.x) + conv) - vec3(3))-vec3(1), 0.0, 1.0);
	//col = col*col*(3.0-2.0*col);
	return c.z * mix(vec3(1), col, c.y);
}

// ======================================= BOUNDING VOLUMES ===================================//

bool intersects_bv(vec3 ro, vec3 rd, vec3 bv_pos, float r){
	bv_pos.z = -bv_pos.z;
	float t = dot(bv_pos-ro, rd);
	vec3 p = ro + rd*t;
	float y = distance(bv_pos, p);
	return y <= (r+TORUS_THICKNESS);
}

bool[MAX_OBJECTS] get_bv_hits(vec3 ro, vec3 rd){
	bool[MAX_OBJECTS] hits;
	for(int i = 0; i < num_segments; i++){
		hits[i] = intersects_bv(ro, rd, center[i], length(radius_vec[i]));
	}
	return hits;
}

int num_hits(bool[MAX_OBJECTS] hits){
	int n = 0;
	for(int i = 0; i < num_segments; i++){
		n = hits[i] ? n+1 : n;
	}
	return n;
}

// ======================================== SDF OPERATIONS =====================================//

float axisX(vec3 p, float c) {
  return length(p.yz)-c;
}
float axisY(vec3 p, float c) {
  return length(p.xz)-c;
}
float axisZ(vec3 p, float c) {
  return length(p.xy)-c;
}

float box(vec3 p, vec3 b){
	vec3 q = abs(p) - b;
	return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}

float torusSegment(vec3 p, float an, float r, float d){
	vec2 sc = vec2(sin(an), cos(an));
	p.x = abs(p.x);
	float k = (sc.y*p.x>sc.x*p.y) ? dot(p.xy,sc) : length(p.xy);
	return sqrt(dot(p,p) + r*r - 2.0*r*k) - d;
}

float frame(vec3 ro, vec3 pl, vec3 rd){
	float tx = (pl.x-ro.x)/rd.x;
	float ty = (pl.y-ro.y)/rd.y;
	float tz = (pl.z-ro.z)/rd.z;

	tx = (tx >= 0) ? tx-0.001*tx : MAX_DIST;
	ty = (ty >= 0) ? ty-0.001*ty : MAX_DIST;
	tz = (tz >= 0) ? tz-0.001*tz : MAX_DIST;

	return min(min(tx, ty), tz);
}

vec2 map(vec3 p, bool[MAX_OBJECTS] hits, vec3 rd){
	float mate = 0.0;
	float dist = 10000000.0;
	float axes = min(min(axisX(p, 0.05), axisY(p, 0.05)), axisZ(p, 0.05));
	dist = min(dist, axes);

	/*float a = 100.0;
	vec3 t = vec3(p.x, p.y, p.z) - vec3(a+planes.x-1., a+planes.y-.5, -a-planes.z+.5);
	float planes_box = -box(t, vec3(a));
	mate = (planes_box < dist) ? -1 : mate;
	dist = min(dist, planes_box);*/

	float torus;
	for(int i = 0; i < num_segments; i++){
		if(hits[i]){
			float alpha = -2.*angle[i];
			float an = (PI/2.)-(alpha/2.);

			vec3 q = vec3(p.x, p.y, -p.z) - center[i];
			vec3 abc = - normalize(radius_vec[i]); // x unit vector -> abc
			vec3 ghi = normal[i]; // z unit vector -> ghi
			vec3 def = normalize(cross(normal[i], radius_vec[i])); // y unit vector -> def

			mat3x3 rota = mat3x3(   abc.x, def.x, ghi.x,
									abc.y, def.y, ghi.y,
									abc.z, def.z, ghi.z   );

			q = rota * q;
			q = rot(q, vec3(0, 0, 1), an);

			torus = torusSegment(q, angle[i], length(radius_vec[i]), TORUS_THICKNESS); mate = (torus < dist) ? float(i+1) : mate; dist = min(dist, torus);
		}
	}
	vec3 pl = vec3(planes.x-1, planes.y-0.5, -planes.z+0.5);
	float planes_frame = frame(p, pl, rd);
	mate = (planes_frame < dist) ? -1 : mate;
	dist = min(dist, planes_frame);

	return vec2(dist, mate);
}

float map_toruses(vec3 p, vec3 ro, vec3 rd){
	float dist = 10000000.0;
	for(int i = 0; i < num_segments; i++){
		vec3 c = center[i];
		if(intersects_bv(ro, rd, center[i], length(radius_vec[i]))){
			float alpha = -2.*angle[i];
			float an = (PI/2.)-(alpha/2.);

			vec3 q = vec3(p.x, p.y, -p.z) - center[i];
			vec3 abc = - normalize(radius_vec[i]); // x unit vector -> abc
			vec3 ghi = normal[i]; // z unit vector -> ghi
			vec3 def = normalize(cross(normal[i], radius_vec[i])); // y unit vector -> def

			mat3x3 rota = mat3x3(   abc.x, def.x, ghi.x,
									abc.y, def.y, ghi.y,
									abc.z, def.z, ghi.z   );

			q = rota * q;
			q = rot(q, vec3(0, 0, 1), an);

			dist = min(dist, torusSegment(q, angle[i], length(radius_vec[i]), TORUS_THICKNESS));
		}
	}
	return dist;
}

vec3 nor(vec3 p, bool[MAX_OBJECTS] bv_hits, vec3 rd){
	vec2 e = vec2(0.001, 0);
	float d = map(p, bv_hits, rd).x;
	vec3 n = d - vec3(map(p-e.xyy, bv_hits, rd).x,
					map(p-e.yxy, bv_hits, rd).x,
					map(p-e.yyx, bv_hits, rd).x);

	return normalize(n);
}

float shadow(vec3 p, vec3 light_dir, bool[MAX_OBJECTS] bv_hits){
	bool hit = false;
	vec3 ray = p;
	for(int i = 0; i < MAX_STEPS; i++){
		float d = map_toruses(ray, p, light_dir);
		hit = d < 0.1;
		ray += light_dir * d;
		if(distance(ray, p) > MAX_DIST) break;
	}
	return hit ? 0.3 : 1.0;
}

// ================================= MAIN FUNCTION; INCLUDES RAYMARCH =============================//

void main(){
	vec2 uv = (gl_FragCoord.xy-0.5*u_resolution)/u_resolution.y;
	vec2 mouse = (u_mouse.xy-0.5*u_resolution)/u_resolution.y;
	float zoom = u_mouse.w;

	// camera setup, including zoom and rotation
	vec3 ro = vec3(0, 2, -8+4.*zoom);
	vec3 rd = normalize(vec3(uv.x, uv.y, 1));
	ro = rot(ro, vec3(0,1,0), 5.*mouse.x);
	rd = rot(rd, vec3(0,1,0), 5.*mouse.x);
	vec3 rot_ax_y = cross(vec3(0,1,0), vec3(sin(5.*mouse.x), 0, cos(5.*mouse.x)));
	ro = rot(ro, rot_ax_y, 2.*mouse.y);
	rd = rot(rd, rot_ax_y, 2.*mouse.y);

	// generating bounding volume hit list
	bool[MAX_OBJECTS] bv_hits = get_bv_hits(ro, rd);

	// ray march
	vec3 p = ro;
	bool hit = false;
	float mate = -1.0;
	for(int i = 0 ; i < MAX_STEPS && !hit; i++){
		vec2 scene = map(p, bv_hits, rd);
		float d = scene.x;
		mate = scene.y;
		hit = d < HIT_DIST;
		if(distance(p, ro) > MAX_DIST) break;
		p += rd*d;
	}

	// coloring by material
	vec3 col = vec3(0.2);
	vec3 n = nor(p, bv_hits, rd);
	if(hit){
		if(mate == -1.0){
			n.xy = -n.xy;
			col = 0.9*(0.5*dot(normalize(vec3(10, 2, -5)-p), n)+vec3(0.5));
			col *= shadow(p, n, bv_hits);
		}
		else if(mate == 0.0){
			col = vec3(0.05);
		}
		else if(mate > 0.0){
			vec3 base = hsv2rgb(vec3((mate - 1.0)/25, 0.9, 0.8));
			vec3 r = reflect(rd, n);
			float shadow = map(p+r, bv_hits, rd).x + 0.1;
			float factor = shadow*length(0.5*sin(r*3.)+0.5)/sqrt(2.);
			col = mix(base, base + vec3(0.1, 0.1, 0), factor) + pow(factor*0.7, 6.);
		}
	}
	fragColor = vec4(col, 1.0);
	// gamma correction, approximated by square root
	fragColor = sqrt(fragColor);
}
