pub type Complex = nalgebra::Complex<f32>;
pub type Vector3 = nalgebra::Vector3<f32>;
pub type Operator = nalgebra::Matrix2<Complex>;
pub type SpinState = nalgebra::Vector2<Complex>;

pub const SZ_POSITIVE_STATE: SpinState =
    SpinState::new(Complex::new(1.0, 0.0), Complex::new(0.0, 0.0));

pub fn b_field(theta: f32, b_field_strength: f32) -> Vector3 {
    Vector3::new(theta.sin(), 0., theta.cos()) * b_field_strength
}

const H_BAR: f32 = 2.;

const SX_OPERATOR: Operator = Operator::new(
    Complex::new(0., 0.),
    Complex::new(1., 0.),
    Complex::new(1., 0.),
    Complex::new(0., 0.),
);

const SY_OPERATOR: Operator = Operator::new(
    Complex::new(0., 0.),
    Complex::new(0., -1.),
    Complex::new(0., 1.),
    Complex::new(0., 0.),
);

const SZ_OPERATOR: Operator = Operator::new(
    Complex::new(1., 0.),
    Complex::new(0., 0.),
    Complex::new(0., 0.),
    Complex::new(-1., 0.),
);

fn expectation(state: SpinState, op: Operator) -> f32 {
    (state.adjoint() * op * state).into_scalar().re
}

/// e^(it)
fn expi(t: f32) -> Complex {
    Complex::new(t.cos(), t.sin())
}

pub fn psi(theta: f32, b_field_strength: f32, time: f32) -> SpinState {
    // Magnitude of energy (same for both states)
    let energy = b_field_strength * H_BAR / 2.0;
    let omega = energy / H_BAR;

    // Energy eigenstates
    let psi_1 = SpinState::new((theta.cos() + 1.0).into(), theta.sin().into());
    let psi_2 = SpinState::new((theta.cos() - 1.0).into(), theta.sin().into());

    (psi_1 * expi(omega * time) - psi_2 * expi(-omega * time)) / Complex::from(2.)
}

pub fn spin_expectation(theta: f32, b_field_strength: f32, time: f32) -> Vector3 {
    let wave = psi(theta, b_field_strength, time);
    Vector3::new(
        expectation(wave, SX_OPERATOR),
        expectation(wave, SY_OPERATOR),
        expectation(wave, SZ_OPERATOR),
    )
}

pub fn spin_expectation_analytical(theta: f32, b_field_strength: f32, time: f32) -> Vector3 {
    let energy = b_field_strength * H_BAR / 1.0;
    let omega = energy / H_BAR;

    let x = (H_BAR / 2.0) * (2. * theta).sin() * (1. - 2.0 * (-omega * time).cos()) / 2.0;

    Vector3::new(x, 0., 0.)
}
