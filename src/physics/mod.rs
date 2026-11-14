use egui::{Pos2, Rect, Vec2};

/// Universal cubic Bézier constant for circle and continuous curvature approximation:
/// kappa = 4 * (sqrt(2) - 1) / 3 ≈ 0.55228475
pub const KAPPA: f32 = 0.55228475;

/// Computes a point on a cubic Bézier curve at parameter t in [0.0, 1.0].
pub fn cubic_bezier_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Pos2::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

/// Generates a smooth squircle path (continuous curvature rounded rectangle)
/// using cubic Bézier segments with kappa = 0.55228475.
pub fn generate_squircle_points(rect: Rect, corner_radius: f32, segments: usize) -> Vec<Pos2> {
    let r = corner_radius.min(rect.width() * 0.5).min(rect.height() * 0.5);
    if r <= 0.0 {
        return vec![
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
    }

    let k = KAPPA * r;
    let mut points = Vec::with_capacity(segments * 4 + 8);
    let steps = segments.max(4);

    // Corner 1: Top-Right (starting at top edge (right - r, top))
    let p0 = Pos2::new(rect.right() - r, rect.top());
    let p1 = Pos2::new(rect.right() - r + k, rect.top());
    let p2 = Pos2::new(rect.right(), rect.top() + r - k);
    let p3 = Pos2::new(rect.right(), rect.top() + r);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    // Corner 2: Bottom-Right
    let p0 = Pos2::new(rect.right(), rect.bottom() - r);
    let p1 = Pos2::new(rect.right(), rect.bottom() - r + k);
    let p2 = Pos2::new(rect.right() - r + k, rect.bottom());
    let p3 = Pos2::new(rect.right() - r, rect.bottom());
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    // Corner 3: Bottom-Left
    let p0 = Pos2::new(rect.left() + r, rect.bottom());
    let p1 = Pos2::new(rect.left() + r - k, rect.bottom());
    let p2 = Pos2::new(rect.left(), rect.bottom() - r + k);
    let p3 = Pos2::new(rect.left(), rect.bottom() - r);
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    // Corner 4: Top-Left
    let p0 = Pos2::new(rect.left(), rect.top() + r);
    let p1 = Pos2::new(rect.left(), rect.top() + r - k);
    let p2 = Pos2::new(rect.left() + r - k, rect.top());
    let p3 = Pos2::new(rect.left() + r, rect.top());
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    points
}

/// Generates a top-bezel anchored Notch geometry with negative-curvature ears
/// transitioning smoothly from the top display edge into the rounded notch body.
pub fn generate_notch_path_points(
    rect: Rect,
    ear_radius: f32,
    corner_radius: f32,
    segments: usize,
) -> Vec<Pos2> {
    let mut points = Vec::new();
    let steps = segments.max(4);
    let ear = ear_radius.max(4.0);
    let r = corner_radius.min(rect.height() * 0.5);
    let k_ear = KAPPA * ear;
    let k_r = KAPPA * r;

    // Start at top screen bezel to the left of the notch ear
    points.push(Pos2::new(rect.left() - ear, rect.top()));

    // Left Ear: negative curvature from bezel downwards
    let p0 = Pos2::new(rect.left() - ear, rect.top());
    let p1 = Pos2::new(rect.left() - ear + k_ear, rect.top());
    let p2 = Pos2::new(rect.left(), rect.top() + ear - k_ear);
    let p3 = Pos2::new(rect.left(), rect.top() + ear);
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    // Left side down to bottom-left corner
    let p0 = Pos2::new(rect.left(), rect.bottom() - r);
    let p1 = Pos2::new(rect.left(), rect.bottom() - r + k_r);
    let p2 = Pos2::new(rect.left() + r - k_r, rect.bottom());
    let p3 = Pos2::new(rect.left() + r, rect.bottom());
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    // Bottom-right corner
    let p0 = Pos2::new(rect.right() - r, rect.bottom());
    let p1 = Pos2::new(rect.right() - r + k_r, rect.bottom());
    let p2 = Pos2::new(rect.right(), rect.bottom() - r + k_r);
    let p3 = Pos2::new(rect.right(), rect.bottom() - r);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    // Right side up to right ear
    let p0 = Pos2::new(rect.right(), rect.top() + ear);
    let p1 = Pos2::new(rect.right(), rect.top() + ear - k_ear);
    let p2 = Pos2::new(rect.right() + ear - k_ear, rect.top());
    let p3 = Pos2::new(rect.right() + ear, rect.top());
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        points.push(cubic_bezier_point(p0, p1, p2, p3, t));
    }

    // Close along the top screen boundary
    points.push(Pos2::new(rect.left() - ear, rect.top()));
    points
}

/// Reverse-engineered NotchPixel analytical multi-frequency damped spring solver:
/// x(t) = 1.0 - e^{-(t / 16.0)^2} * [1.4 * A * sin(4.0 * t - 0.5) + 0.18 * A * sin(7.5 * t)]
pub fn notchpixel_spring_solve(t_sec: f32, amplitude: f32) -> f32 {
    let t_norm = t_sec.max(0.0);
    let decay = (-((t_norm * 1000.0 / 16.0).powi(2) * 0.0001)).exp();
    let osc1 = 1.4 * amplitude * (4.0 * t_norm - 0.5).sin();
    let osc2 = 0.18 * amplitude * (7.5 * t_norm).sin();
    1.0 - decay * (osc1 + osc2)
}

/// Numerical 2nd-order damped harmonic oscillator for real-time interactive physics.
#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub current: f32,
    pub target: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl Spring {
    pub fn new(initial: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            velocity: 0.0,
            stiffness,
            damping,
        }
    }

    /// Advances the spring simulation by dt (seconds) using semi-implicit Euler integration.
    pub fn update(&mut self, dt: f32) {
        let clamped_dt = dt.clamp(0.001, 0.050);
        let displacement = self.current - self.target;
        let spring_force = -self.stiffness * displacement;
        let damping_force = -self.damping * self.velocity;
        let acceleration = spring_force + damping_force;

        self.velocity += acceleration * clamped_dt;
        self.current += self.velocity * clamped_dt;

        // Snap to target when settled within threshold
        if (self.current - self.target).abs() < 0.1 && self.velocity.abs() < 0.2 {
            self.current = self.target;
            self.velocity = 0.0;
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < 0.1 && self.velocity.abs() < 0.2
    }
}

/// Morphing state coordinator for Dynamic Island dimensions, corner radii, and opacity.
#[derive(Debug, Clone)]
pub struct IslandMorphSpring {
    pub width: Spring,
    pub height: Spring,
    pub corner_radius: Spring,
    pub opacity: Spring,
    pub elapsed_time: f32,
}

impl IslandMorphSpring {
    pub fn new(initial_w: f32, initial_h: f32, initial_r: f32) -> Self {
        // High stiffness and critically damped for snappy, premium feel
        Self {
            width: Spring::new(initial_w, 240.0, 22.0),
            height: Spring::new(initial_h, 240.0, 22.0),
            corner_radius: Spring::new(initial_r, 220.0, 20.0),
            opacity: Spring::new(1.0, 200.0, 20.0),
            elapsed_time: 0.0,
        }
    }

    pub fn set_target_geometry(&mut self, target_w: f32, target_h: f32, target_r: f32) {
        self.width.set_target(target_w);
        self.height.set_target(target_h);
        self.corner_radius.set_target(target_r);
    }

    pub fn set_target_opacity(&mut self, target_alpha: f32) {
        self.opacity.set_target(target_alpha);
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
        self.width.update(dt);
        self.height.update(dt);
        self.corner_radius.update(dt);
        self.opacity.update(dt);
    }

    pub fn current_size(&self) -> Vec2 {
        Vec2::new(self.width.current, self.height.current)
    }

    pub fn is_settled(&self) -> bool {
        self.width.is_settled()
            && self.height.is_settled()
            && self.corner_radius.is_settled()
            && self.opacity.is_settled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_bezier_endpoints() {
        let p0 = Pos2::new(0.0, 0.0);
        let p1 = Pos2::new(0.0, 5.0);
        let p2 = Pos2::new(5.0, 10.0);
        let p3 = Pos2::new(10.0, 10.0);

        let start = cubic_bezier_point(p0, p1, p2, p3, 0.0);
        let end = cubic_bezier_point(p0, p1, p2, p3, 1.0);

        assert!((start.x - 0.0).abs() < 1e-5);
        assert!((start.y - 0.0).abs() < 1e-5);
        assert!((end.x - 10.0).abs() < 1e-5);
        assert!((end.y - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_squircle_points_generation() {
        let rect = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(200.0, 100.0));
        let pts = generate_squircle_points(rect, 20.0, 8);
        assert!(pts.len() >= 32);
        for pt in pts {
            assert!(rect.expand(1.0).contains(pt));
        }
    }

    #[test]
    fn test_spring_convergence() {
        let mut spring = Spring::new(0.0, 200.0, 20.0);
        spring.set_target(100.0);
        for _ in 0..300 {
            spring.update(0.016);
        }
        assert!((spring.current - 100.0).abs() < 0.2);
        assert!(spring.is_settled());
    }

    #[test]
    fn test_island_morph_spring() {
        let mut morph = IslandMorphSpring::new(180.0, 36.0, 18.0);
        morph.set_target_geometry(460.0, 220.0, 24.0);
        for _ in 0..300 {
            morph.update(0.016);
        }
        let sz = morph.current_size();
        assert!((sz.x - 460.0).abs() < 0.5);
        assert!((sz.y - 220.0).abs() < 0.5);
        assert!((morph.corner_radius.current - 24.0).abs() < 0.5);
    }
}
