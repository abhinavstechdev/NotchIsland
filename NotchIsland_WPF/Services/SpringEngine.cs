using System;

namespace NotchIsland.Services
{
    /// <summary>
    /// Exact 2nd-order analytical damped harmonic oscillator physics engine.
    /// Replicates NotchPixel R.RK and R.Rs analytical closed-form dynamics.
    /// </summary>
    public readonly struct SpringParams
    {
        public double Response { get; }
        public double DampingRatio { get; }
        public double Omega0 { get; }
        public double Stiffness { get; }
        public double DampingCoeff { get; }

        public SpringParams(double response, double dampingRatio)
        {
            Response = Math.Max(0.01, response);
            DampingRatio = Math.Max(0.0, dampingRatio);
            Omega0 = (2.0 * Math.PI) / Response;
            Stiffness = Omega0 * Omega0; // mass m = 1.0
            DampingCoeff = 2.0 * Omega0 * DampingRatio;
        }

        public static SpringParams Create(double response, double dampingRatio) => new(response, dampingRatio);

        // Standard Presets from NotchPixel R.Rs
        public static SpringParams Bc => new(0.42, 0.80);  // Card width & height morphing
        public static SpringParams Bh => new(0.45, 1.00);  // Critically damped settling & ears
        public static SpringParams Bs => new(0.30, 1.00);  // Snappy collapse, corner radius, alpha
        public static SpringParams BI => new(0.38, 0.80);  // Fast pill expansion pop
        public static SpringParams Bu => new(0.40, 0.70);  // Bouncy HUD pop
        public static SpringParams BS => new(0.40, 0.85);  // Subpanel layout morph
        public static SpringParams BK => new(0.40, 1.00);  // Critically damped tab switch
        public static SpringParams BV => new(0.55, 0.825); // Large card fluid expansion
        public static SpringParams ElasticImpact => new(0.25, 0.50); // Click/tap bounce impulse
        public static SpringParams QuickSnap => new(0.22, 1.00);    // Quick snap toggle
        public static SpringParams TabSlide => new(0.42, 0.86);     // vaa channel tab slide
        public static SpringParams TabFade => new(0.38, 1.00);      // vae channel tab fade

        public static SpringParams CardMorph => Bc;
        public static SpringParams Snappy => Bs;
        public static SpringParams Fluid => BV;
    }

    public class AnalyticalSpring : Spring
    {
        public AnalyticalSpring(double initialValue, SpringParams springParams) : base(initialValue, springParams) { }
    }

    public class Spring
    {
        public double Current { get; set; }
        public double Value { get => Current; set => Current = value; }
        public double Target { get; set; }
        public double Velocity { get; set; }
        public SpringParams Params { get; set; }
        public bool IsAtRest { get; private set; }

        public Spring(double initialValue, SpringParams springParams)
        {
            Current = initialValue;
            Target = initialValue;
            Velocity = 0.0;
            Params = springParams;
            IsAtRest = true;
        }

        public void SnapTo(double value)
        {
            Current = value;
            Target = value;
            Velocity = 0.0;
            IsAtRest = true;
        }

        public void SetTarget(double target)
        {
            if (Math.Abs(Target - target) > 0.0001)
            {
                Target = target;
                IsAtRest = false;
            }
        }

        public void SetTarget(double target, SpringParams springParams)
        {
            Params = springParams;
            SetTarget(target);
        }

        public double Update(double dt)
        {
            // Safeguard against frame freezes or sleep
            if (dt > 0.25)
            {
                Current = Target;
                Velocity = 0.0;
                IsAtRest = true;
                return Current;
            }

            if (dt <= 0.0) return Current;

            double x0 = Current - Target;
            double v0 = Velocity;

            // Rest condition check
            if (Math.Abs(x0) < 0.05 && Math.Abs(v0) < 1.0)
            {
                Current = Target;
                Velocity = 0.0;
                IsAtRest = true;
                return Current;
            }

            IsAtRest = false;
            double zeta = Params.DampingRatio;
            double omega0 = Params.Omega0;
            double xt;
            double vt;

            if (zeta < 0.999)
            {
                // Underdamped regime
                double omegaD = omega0 * Math.Sqrt(1.0 - (zeta * zeta));
                double decay = Math.Exp(-zeta * omega0 * dt);
                double cosD = Math.Cos(omegaD * dt);
                double sinD = Math.Sin(omegaD * dt);

                xt = decay * (x0 * cosD + ((v0 + zeta * omega0 * x0) / omegaD) * sinD);
                vt = decay * (v0 * cosD - ((zeta * omega0 * v0 + omega0 * omega0 * x0) / omegaD) * sinD);
            }
            else if (zeta > 1.001)
            {
                // Overdamped regime
                double alpha = omega0 * Math.Sqrt((zeta * zeta) - 1.0);
                double gamma1 = -zeta * omega0 + alpha;
                double gamma2 = -zeta * omega0 - alpha;
                double c2 = (v0 - gamma1 * x0) / (gamma2 - gamma1);
                double c1 = x0 - c2;
                double e1 = Math.Exp(gamma1 * dt);
                double e2 = Math.Exp(gamma2 * dt);

                xt = c1 * e1 + c2 * e2;
                vt = c1 * gamma1 * e1 + c2 * gamma2 * e2;
            }
            else
            {
                // Critically damped regime (0.999 <= zeta <= 1.001)
                double decay = Math.Exp(-omega0 * dt);
                xt = decay * (x0 + (v0 + omega0 * x0) * dt);
                vt = decay * (v0 - omega0 * (v0 + omega0 * x0) * dt);
            }

            Current = Target + xt;
            Velocity = vt;

            if (Math.Abs(Current - Target) < 0.05 && Math.Abs(Velocity) < 1.0)
            {
                Current = Target;
                Velocity = 0.0;
                IsAtRest = true;
            }

            return Current;
        }
    }

    public class Spring2D
    {
        public Spring X { get; }
        public Spring Y { get; }

        public bool IsAtRest => X.IsAtRest && Y.IsAtRest;

        public Spring2D(double initialX, double initialY, SpringParams springParams)
        {
            X = new Spring(initialX, springParams);
            Y = new Spring(initialY, springParams);
        }

        public void SetTarget(double targetX, double targetY)
        {
            X.Target = targetX;
            Y.Target = targetY;
        }

        public void SnapTo(double targetX, double targetY)
        {
            X.SnapTo(targetX);
            Y.SnapTo(targetY);
        }

        public (double x, double y) Update(double dt)
        {
            return (X.Update(dt), Y.Update(dt));
        }
    }

    public static class MotionPhysics
    {
        /// <summary>
        /// Exponential smoothing function v(c, t, speed, dt) = c + (t - c)(1 - e^(-dt * speed))
        /// Used in NotchPixel X.XE for media active/inactive blends (speed 8.0/14.0).
        /// </summary>
        public static double ExponentialSmooth(double current, double target, double speed, double dt)
        {
            if (dt <= 0.0) return current;
            return current + (target - current) * (1.0 - Math.Exp(-dt * speed));
        }

        /// <summary>
        /// Tap / click elastic impulse waveform from XE.cs:9987 (total 450ms).
        /// Attack (t &lt; 60ms): linear ramp to 1.0.
        /// Decay (t &gt;= 60ms): e^(-(t-0.06)/0.09) * cos((t-0.06)*18.0).
        /// </summary>
        public static double ButtonTapImpulse(double timeSeconds)
        {
            if (timeSeconds < 0.0 || timeSeconds > 0.45) return 0.0;
            if (timeSeconds < 0.06)
            {
                return timeSeconds / 0.06;
            }
            double tDecay = timeSeconds - 0.06;
            return Math.Exp(-tDecay / 0.09) * Math.Cos(tDecay * 18.0);
        }

        /// <summary>
        /// Audio progress bar wave ripple from XE.cs:7221.
        /// Segment dot spacing = 11px, hover bulge Gaussian, harmonic sine ripple.
        /// </summary>
        public static double ProgressBarWave(int segmentIndex, double timeSeconds, double activeRatio)
        {
            return Math.Sin(timeSeconds * 4.0 - segmentIndex * 0.5) * 1.4 * activeRatio;
        }

        /// <summary>
        /// Gaussian hover bulge intensity for progress bar scrubber.
        /// </summary>
        public static double HoverBulge(double x, double hoverX, double intensity)
        {
            double diff = (x - hoverX) / 16.0;
            return Math.Exp(-(diff * diff)) * intensity;
        }
    }
}
