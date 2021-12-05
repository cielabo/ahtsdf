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

float torusSegment(vec3 p, float an, float r, float d){
	vec2 sc = vec2(sin(an), cos(an));
	p.x = abs(p.x);
	float k = (sc.y*p.x>sc.x*p.y) ? dot(p.xy,sc) : length(p.xy);
	return sqrt(dot(p,p) + r*r - 2.0*r*k) - d;
}
float discSegment(vec3 p, float an, float r, float h) {
	p.x = abs(p.x);
    vec2 sc = vec2(sin(an), cos(an));
    float dist = (sc.y*p.x>sc.x*p.y) ? distance(clamp(dot(p.xy, sc), 0., r)*sc, p.xy): length(p.xy)-r;
    vec2 w = vec2(dist, abs(p.z) - h);
    return min(max(w.x,w.y),0.0) + length(max(w,0.0));
}

vec2 map(vec3 p, bool[MAX_OBJECTS] hits){
	float mate = 0.0;
	float dist = 10000000.0;
	float axes = min(min(axisX(p, 0.05), axisY(p, 0.05)), axisZ(p, 0.05));
	dist = min(dist, axes);

	vec3 a = p;
	float point;
    a.x = mod(a.x+0.5, 1.)-0.5;
    point = length(a)-0.08;
    mate = (point < dist) ? -1.0 : mate;
    dist = min(dist, point);
    a = p;
    a.y = mod(a.y+0.5, 1.)-0.5;
    point = length(a)-0.08;
    mate = (point < dist) ? -3.0 : mate;
    dist = min(dist, point);
    a = p;
    a.z = mod(a.z+0.5, 1.)-0.5;
    point = length(a)-0.08;
    mate = (point < dist) ? -2.0 : mate;
    dist = min(dist, point);

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

			torus = discSegment(q, angle[i], length(radius_vec[i]), 0.5*TORUS_THICKNESS)-.02;
			mate = (torus < dist) ? float(i+1) : mate;
			dist = min(dist, torus);
		}
	}
	return vec2(dist, mate);
}

vec3 nor(vec3 p, bool[MAX_OBJECTS] bv_hits){
	vec2 e = vec2(0.001, 0);
	float d = map(p, bv_hits).x;
	vec3 n = d - vec3(map(p-e.xyy, bv_hits).x, 
					map(p-e.yxy, bv_hits).x, 
					map(p-e.yyx, bv_hits).x);

	return normalize(n);
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
		vec2 scene = map(p, bv_hits);
		float d = scene.x;
		mate = scene.y;
		hit = d < HIT_DIST;
		p += rd*d;
		if(distance(p, ro) > MAX_DIST) break;
	}

	// coloring by material
	vec3 col = vec3(0.15);
	vec3 n = nor(p, bv_hits);
	if(hit){
		if(mate == 0.0){
			col = vec3(0.05);
		}
		else if(mate == -1.0){
			col = vec3(1.0, 0.1, 0.0);
		}
		else if(mate == -2.0){
			col = vec3(1.0, 1.0, 0.0);
		}
		else if(mate == -3.0){
			col = vec3(0.8, 0.8, 0.9);
		}
		else if(mate > 0.0){
			vec3 base = hsv2rgb(vec3((mate - 1.0)/25, 0.9, 0.6));
			vec3 highlight = hsv2rgb(vec3((mate - 1.0)/25, 0.8, 0.8));
			col = mix(base, highlight, 0.5*dot(n, normalize(vec3(1)))+0.5);
			col += pow(max(dot(reflect(rd, n), normalize(vec3(1))), 0), 10) * highlight * 1.5;
		}
	}
	fragColor = vec4(col, 1.0);
	// gamma correction, approximated by square root
	fragColor = sqrt(fragColor);
}
