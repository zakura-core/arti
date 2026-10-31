//! Define the congestion control parameters needed for the algorithms.
//!
//! All of these values are taken from the consensus. And so the details of these values can be
//! found in section 6.5.1. of proposal 324.

use caret::caret_int;

use tor_config::ConfigBuildError;
use tor_units::Percentage;

/// Require a field, allocating its diagnostic only if it is missing.
fn required_field<T>(value: Option<T>, field: &'static str) -> Result<T, ConfigBuildError> {
    value.ok_or_else(|| ConfigBuildError::MissingField {
        field: field.into(),
    })
}

/// Fixed window parameters that are for the SENDME v0 world of fixed congestion window.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, getset::CopyGetters)]
pub struct FixedWindowParams {
    /// Circuit window starting point. From the "circwindow" param.
    #[getset(get_copy = "pub")]
    circ_window_start: u16,
    /// Circuit window minimum value.
    #[getset(get_copy = "pub")]
    circ_window_min: u16,
    /// Circuit window maximum value.
    #[getset(get_copy = "pub")]
    circ_window_max: u16,
}
/// Builder for [`FixedWindowParams`].
#[derive(Clone, Default)]
pub struct FixedWindowParamsBuilder {
    /// The configured `circ_window_start` value.
    circ_window_start: Option<u16>,
    /// The configured `circ_window_min` value.
    circ_window_min: Option<u16>,
    /// The configured `circ_window_max` value.
    circ_window_max: Option<u16>,
}

impl FixedWindowParamsBuilder {
    /// Set the [`FixedWindowParams::circ_window_start`] value.
    pub fn circ_window_start(&mut self, value: u16) -> &mut Self {
        self.circ_window_start = Some(value);
        self
    }

    /// Set the [`FixedWindowParams::circ_window_min`] value.
    pub fn circ_window_min(&mut self, value: u16) -> &mut Self {
        self.circ_window_min = Some(value);
        self
    }

    /// Set the [`FixedWindowParams::circ_window_max`] value.
    pub fn circ_window_max(&mut self, value: u16) -> &mut Self {
        self.circ_window_max = Some(value);
        self
    }

    /// Build the parameters, requiring every field to be set.
    pub fn build(&self) -> Result<FixedWindowParams, ConfigBuildError> {
        Ok(FixedWindowParams {
            circ_window_start: required_field(self.circ_window_start, "circ_window_start")?,
            circ_window_min: required_field(self.circ_window_min, "circ_window_min")?,
            circ_window_max: required_field(self.circ_window_max, "circ_window_max")?,
        })
    }
}

impl FixedWindowParams {
    /// Return a fresh, default builder.
    pub fn builder() -> FixedWindowParamsBuilder {
        FixedWindowParamsBuilder::default()
    }
}

impl tor_config::load::Builder for FixedWindowParamsBuilder {
    type Built = FixedWindowParams;

    fn build(&self) -> Result<FixedWindowParams, ConfigBuildError> {
        FixedWindowParamsBuilder::build(self)
    }
}

impl FixedWindowParams {
    #[cfg(test)]
    // These have been copied from C-tor.
    pub(crate) fn defaults_for_tests() -> Self {
        Self {
            circ_window_start: 1000,
            circ_window_min: 100,
            circ_window_max: 1000,
        }
    }
}

/// Vegas queuing parameters taken from the consensus only which are different depending if the
/// circuit is an onion service one, an exit or used for SBWS.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, getset::CopyGetters)]
pub struct VegasQueueParams {
    /// Alpha parameter is used to know when to increase the window.
    #[getset(get_copy = "pub")]
    alpha: u32,
    /// Beta parameter is used to know when to decrease the window
    #[getset(get_copy = "pub")]
    beta: u32,
    /// Delta parameter is used as an indicator to drop the window to this considering the current
    /// BDP value and increment.
    #[getset(get_copy = "pub")]
    delta: u32,
    /// Gamma parameter is only used in slow start and used to know when to increase or adjust the
    /// window with the BDP.
    #[getset(get_copy = "pub")]
    gamma: u32,
    /// Parameter describe the RFC3742 'cap', after which congestion window increments are reduced.
    /// INT32_MAX disables
    #[getset(get_copy = "pub")]
    ss_cwnd_cap: u32,
}

/// Used when we parse at once all the specific circuit type vegas queue parameters. They are
/// bundled in a 5-tuple and transformed with this.
impl From<(u32, u32, u32, u32, u32)> for VegasQueueParams {
    fn from(v: (u32, u32, u32, u32, u32)) -> Self {
        Self {
            alpha: v.0,
            beta: v.1,
            delta: v.2,
            gamma: v.3,
            ss_cwnd_cap: v.4,
        }
    }
}

/// Vegas algorithm parameters taken from the consensus.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, getset::Getters, getset::CopyGetters)]
pub struct VegasParams {
    /// The amount of queued cells that Vegas can tolerate before reacting.
    #[getset(get = "pub")]
    cell_in_queue_params: VegasQueueParams,
    /// A hard-max on the congestion window in Slow Start.
    #[getset(get_copy = "pub")]
    ss_cwnd_max: u32,
    /// This parameter defines the integer number of 'cc_sendme_inc' multiples
    /// of gap allowed between inflight and cwnd, to still declare the cwnd full.
    #[getset(get_copy = "pub")]
    cwnd_full_gap: u32,
    /// This parameter defines a low watermark in percent.
    #[getset(get = "pub")]
    cwnd_full_min_pct: Percentage<u32>,
    /// This parameter governs how often a cwnd must be full.
    #[getset(get_copy = "pub")]
    cwnd_full_per_cwnd: u32,
}
/// Builder for [`VegasParams`].
#[derive(Clone, Default)]
pub struct VegasParamsBuilder {
    /// The configured `cell_in_queue_params` value.
    cell_in_queue_params: Option<VegasQueueParams>,
    /// The configured `ss_cwnd_max` value.
    ss_cwnd_max: Option<u32>,
    /// The configured `cwnd_full_gap` value.
    cwnd_full_gap: Option<u32>,
    /// The configured `cwnd_full_min_pct` value.
    cwnd_full_min_pct: Option<Percentage<u32>>,
    /// The configured `cwnd_full_per_cwnd` value.
    cwnd_full_per_cwnd: Option<u32>,
}

impl VegasParamsBuilder {
    /// Set the [`VegasParams::cell_in_queue_params`] value.
    pub fn cell_in_queue_params(&mut self, value: VegasQueueParams) -> &mut Self {
        self.cell_in_queue_params = Some(value);
        self
    }

    /// Set the [`VegasParams::ss_cwnd_max`] value.
    pub fn ss_cwnd_max(&mut self, value: u32) -> &mut Self {
        self.ss_cwnd_max = Some(value);
        self
    }

    /// Set the [`VegasParams::cwnd_full_gap`] value.
    pub fn cwnd_full_gap(&mut self, value: u32) -> &mut Self {
        self.cwnd_full_gap = Some(value);
        self
    }

    /// Set the [`VegasParams::cwnd_full_min_pct`] value.
    pub fn cwnd_full_min_pct(&mut self, value: Percentage<u32>) -> &mut Self {
        self.cwnd_full_min_pct = Some(value);
        self
    }

    /// Set the [`VegasParams::cwnd_full_per_cwnd`] value.
    pub fn cwnd_full_per_cwnd(&mut self, value: u32) -> &mut Self {
        self.cwnd_full_per_cwnd = Some(value);
        self
    }

    /// Build the parameters, requiring every field to be set.
    pub fn build(&self) -> Result<VegasParams, ConfigBuildError> {
        Ok(VegasParams {
            cell_in_queue_params: required_field(
                self.cell_in_queue_params,
                "cell_in_queue_params",
            )?,
            ss_cwnd_max: required_field(self.ss_cwnd_max, "ss_cwnd_max")?,
            cwnd_full_gap: required_field(self.cwnd_full_gap, "cwnd_full_gap")?,
            cwnd_full_min_pct: required_field(self.cwnd_full_min_pct, "cwnd_full_min_pct")?,
            cwnd_full_per_cwnd: required_field(self.cwnd_full_per_cwnd, "cwnd_full_per_cwnd")?,
        })
    }
}

impl VegasParams {
    /// Return a fresh, default builder.
    pub fn builder() -> VegasParamsBuilder {
        VegasParamsBuilder::default()
    }
}

impl tor_config::load::Builder for VegasParamsBuilder {
    type Built = VegasParams;

    fn build(&self) -> Result<VegasParams, ConfigBuildError> {
        VegasParamsBuilder::build(self)
    }
}

impl VegasParams {
    #[cfg(test)]
    // These have been copied from spec (prop324).
    pub(crate) fn defaults_for_tests() -> Self {
        // The OUTBUF_CELLS size from prop324.
        const OC: u32 = 62;
        Self {
            cell_in_queue_params: (3 * OC, 4 * OC, 5 * OC, 3 * OC, 600).into(),
            ss_cwnd_max: 5000,
            cwnd_full_gap: 4,
            cwnd_full_min_pct: Percentage::new(25),
            cwnd_full_per_cwnd: 1,
        }
    }
}

/// The different congestion control algorithms. Each contain their parameters taken from the
/// consensus.
#[non_exhaustive]
#[derive(Clone, Debug, strum::EnumDiscriminants)]
// No need for the discriminants to be public at the moment,
// and we might want to rename it if it does become public.
#[strum_discriminants(vis(pub(crate)))]
pub enum Algorithm {
    /// Fixed window algorithm.
    FixedWindow(FixedWindowParams),
    /// Vegas algorithm.
    Vegas(VegasParams),
}

impl Algorithm {
    /// Return true if this algorithm can be used along with CGO.
    ///
    /// CGO requires the V1 relay cell format, where every relay command
    /// implies either the presence or absence of a StreamID.
    /// But that format is not compatible with (legacy) stream-level SENDME messages
    /// for flow control.
    pub(crate) fn compatible_with_cgo(&self) -> bool {
        match self {
            Algorithm::FixedWindow(_) => false,
            Algorithm::Vegas(_) => true,
        }
    }
}

caret_int! {
    /// Congestion control algorithm types defined by numerical values. See "cc_alg" in proposal
    /// 324 section 6.5.1 for the supported values.
    ///
    /// This is a i32 so it is the same type as the consensus supported value type.
    pub struct AlgorithmType(i32) {
        /// Fixed window algorithm.
        FIXED_WINDOW = 0,
        /// Vegas algorithm.
        VEGAS = 2,
    }
}

/// The round trip estimator parameters taken from consensus and used to estimate the round trip
/// time on a circuit.
#[non_exhaustive]
#[derive(Clone, Debug, getset::Getters, getset::CopyGetters)]
pub struct RoundTripEstimatorParams {
    /// The "N" parameter in N-EWMA smoothing of RTT and/or bandwidth estimation, specified as a
    /// percentage of the number of SENDME acks in a congestion window.
    ///
    /// A percentage over 100% indicates smoothing with more than one congestion window's worth
    /// of SENDMEs.
    #[getset(get = "pub")]
    ewma_cwnd_pct: Percentage<u32>,
    /// The maximum value of the "N" parameter in N-EWMA smoothing of RTT and/or bandwidth
    /// estimation.
    #[getset(get_copy = "pub")]
    ewma_max: u32,
    /// The maximum value of the "N" parameter in N-EWMA smoothing of RTT and/or bandwidth
    /// estimation but in Slow Start.
    #[getset(get_copy = "pub")]
    ewma_ss_max: u32,
    /// Describes a percentile average between min and current ewma, for use to reset RTT_min, when
    /// the congestion window hits cwnd_min.
    #[getset(get = "pub")]
    rtt_reset_pct: Percentage<u32>,
}
/// Builder for [`RoundTripEstimatorParams`].
#[derive(Clone, Default)]
pub struct RoundTripEstimatorParamsBuilder {
    /// The configured `ewma_cwnd_pct` value.
    ewma_cwnd_pct: Option<Percentage<u32>>,
    /// The configured `ewma_max` value.
    ewma_max: Option<u32>,
    /// The configured `ewma_ss_max` value.
    ewma_ss_max: Option<u32>,
    /// The configured `rtt_reset_pct` value.
    rtt_reset_pct: Option<Percentage<u32>>,
}

impl RoundTripEstimatorParamsBuilder {
    /// Set the [`RoundTripEstimatorParams::ewma_cwnd_pct`] value.
    pub fn ewma_cwnd_pct(&mut self, value: Percentage<u32>) -> &mut Self {
        self.ewma_cwnd_pct = Some(value);
        self
    }

    /// Set the [`RoundTripEstimatorParams::ewma_max`] value.
    pub fn ewma_max(&mut self, value: u32) -> &mut Self {
        self.ewma_max = Some(value);
        self
    }

    /// Set the [`RoundTripEstimatorParams::ewma_ss_max`] value.
    pub fn ewma_ss_max(&mut self, value: u32) -> &mut Self {
        self.ewma_ss_max = Some(value);
        self
    }

    /// Set the [`RoundTripEstimatorParams::rtt_reset_pct`] value.
    pub fn rtt_reset_pct(&mut self, value: Percentage<u32>) -> &mut Self {
        self.rtt_reset_pct = Some(value);
        self
    }

    /// Build the parameters, requiring every field to be set.
    pub fn build(&self) -> Result<RoundTripEstimatorParams, ConfigBuildError> {
        Ok(RoundTripEstimatorParams {
            ewma_cwnd_pct: required_field(self.ewma_cwnd_pct, "ewma_cwnd_pct")?,
            ewma_max: required_field(self.ewma_max, "ewma_max")?,
            ewma_ss_max: required_field(self.ewma_ss_max, "ewma_ss_max")?,
            rtt_reset_pct: required_field(self.rtt_reset_pct, "rtt_reset_pct")?,
        })
    }
}

impl RoundTripEstimatorParams {
    /// Return a fresh, default builder.
    pub fn builder() -> RoundTripEstimatorParamsBuilder {
        RoundTripEstimatorParamsBuilder::default()
    }
}

impl tor_config::load::Builder for RoundTripEstimatorParamsBuilder {
    type Built = RoundTripEstimatorParams;

    fn build(&self) -> Result<RoundTripEstimatorParams, ConfigBuildError> {
        RoundTripEstimatorParamsBuilder::build(self)
    }
}

impl RoundTripEstimatorParams {
    #[cfg(test)]
    // These have been copied from spec (prop324).
    pub(crate) fn defaults_for_tests() -> Self {
        Self {
            ewma_cwnd_pct: Percentage::new(50),
            ewma_max: 10,
            ewma_ss_max: 2,
            rtt_reset_pct: Percentage::new(100),
        }
    }
}

/// The parameters of what constitute a congestion window. This is used by all congestion control
/// algorithms as in it is not specific to an algorithm.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, getset::Getters, getset::CopyGetters)]
pub struct CongestionWindowParams {
    /// Initial size of the congestion window.
    #[getset(get_copy = "pub")]
    cwnd_init: u32,
    /// Percent of cwnd to increment by during slow start.
    #[getset(get = "pub")]
    cwnd_inc_pct_ss: Percentage<u32>,
    /// Number of cells to increment cwnd by during steady state.
    #[getset(get_copy = "pub")]
    cwnd_inc: u32,
    /// Number of times per congestion window to update based on congestion signals.
    #[getset(get_copy = "pub")]
    cwnd_inc_rate: u32,
    /// Minimum congestion window (must be at least sendme_inc)
    #[getset(get_copy = "pub")]
    cwnd_min: u32,
    /// Maximum congestion window
    #[getset(get_copy = "pub")]
    cwnd_max: u32,
    /// The SENDME increment as in the number of cells to ACK with every SENDME. This is coming
    /// from the consensus and negotiated during circuit setup.
    #[getset(get_copy = "pub")]
    sendme_inc: u8,
}
/// Builder for [`CongestionWindowParams`].
#[derive(Clone, Default)]
pub struct CongestionWindowParamsBuilder {
    /// The configured `cwnd_init` value.
    cwnd_init: Option<u32>,
    /// The configured `cwnd_inc_pct_ss` value.
    cwnd_inc_pct_ss: Option<Percentage<u32>>,
    /// The configured `cwnd_inc` value.
    cwnd_inc: Option<u32>,
    /// The configured `cwnd_inc_rate` value.
    cwnd_inc_rate: Option<u32>,
    /// The configured `cwnd_min` value.
    cwnd_min: Option<u32>,
    /// The configured `cwnd_max` value.
    cwnd_max: Option<u32>,
    /// The configured `sendme_inc` value.
    sendme_inc: Option<u8>,
}

impl CongestionWindowParamsBuilder {
    /// Set the [`CongestionWindowParams::cwnd_init`] value.
    pub fn cwnd_init(&mut self, value: u32) -> &mut Self {
        self.cwnd_init = Some(value);
        self
    }

    /// Set the [`CongestionWindowParams::cwnd_inc_pct_ss`] value.
    pub fn cwnd_inc_pct_ss(&mut self, value: Percentage<u32>) -> &mut Self {
        self.cwnd_inc_pct_ss = Some(value);
        self
    }

    /// Set the [`CongestionWindowParams::cwnd_inc`] value.
    pub fn cwnd_inc(&mut self, value: u32) -> &mut Self {
        self.cwnd_inc = Some(value);
        self
    }

    /// Set the [`CongestionWindowParams::cwnd_inc_rate`] value.
    pub fn cwnd_inc_rate(&mut self, value: u32) -> &mut Self {
        self.cwnd_inc_rate = Some(value);
        self
    }

    /// Set the [`CongestionWindowParams::cwnd_min`] value.
    pub fn cwnd_min(&mut self, value: u32) -> &mut Self {
        self.cwnd_min = Some(value);
        self
    }

    /// Set the [`CongestionWindowParams::cwnd_max`] value.
    pub fn cwnd_max(&mut self, value: u32) -> &mut Self {
        self.cwnd_max = Some(value);
        self
    }

    /// Set the [`CongestionWindowParams::sendme_inc`] value.
    pub fn sendme_inc(&mut self, value: u8) -> &mut Self {
        self.sendme_inc = Some(value);
        self
    }

    /// Build the parameters, requiring every field to be set.
    pub fn build(&self) -> Result<CongestionWindowParams, ConfigBuildError> {
        Ok(CongestionWindowParams {
            cwnd_init: required_field(self.cwnd_init, "cwnd_init")?,
            cwnd_inc_pct_ss: required_field(self.cwnd_inc_pct_ss, "cwnd_inc_pct_ss")?,
            cwnd_inc: required_field(self.cwnd_inc, "cwnd_inc")?,
            cwnd_inc_rate: required_field(self.cwnd_inc_rate, "cwnd_inc_rate")?,
            cwnd_min: required_field(self.cwnd_min, "cwnd_min")?,
            cwnd_max: required_field(self.cwnd_max, "cwnd_max")?,
            sendme_inc: required_field(self.sendme_inc, "sendme_inc")?,
        })
    }
}

impl CongestionWindowParams {
    /// Return a fresh, default builder.
    pub fn builder() -> CongestionWindowParamsBuilder {
        CongestionWindowParamsBuilder::default()
    }
}

impl tor_config::load::Builder for CongestionWindowParamsBuilder {
    type Built = CongestionWindowParams;

    fn build(&self) -> Result<CongestionWindowParams, ConfigBuildError> {
        CongestionWindowParamsBuilder::build(self)
    }
}

impl CongestionWindowParams {
    /// Set the `sendme_inc` value.
    ///
    /// This is used to override the default increment value from when this was constructed with a
    /// [`CongestionWindowParamsBuilder`].
    /// Typically the default when built should be from the network parameters from the consensus.
    pub(crate) fn set_sendme_inc(&mut self, inc: u8) {
        self.sendme_inc = inc;
    }

    #[cfg(test)]
    // These have been copied from spec (prop324).
    pub(crate) fn defaults_for_tests() -> Self {
        Self {
            cwnd_init: 4 * 31,
            cwnd_inc_pct_ss: Percentage::new(50),
            cwnd_inc: 31,
            cwnd_inc_rate: 1,
            cwnd_min: 31,
            cwnd_max: u32::MAX,
            sendme_inc: 31,
        }
    }
}

/// Global congestion control parameters taken from consensus. These are per-circuit.
#[non_exhaustive]
#[derive(Clone, Debug, getset::Getters, getset::CopyGetters, getset::MutGetters)]
pub struct CongestionControlParams {
    /// The congestion control algorithm to use.
    #[getset(get = "pub")]
    alg: Algorithm,
    /// Parameters to the fallback fixed-window algorithm, which we use
    /// when the one in `alg` is not supported by a given relay.
    ///
    /// It is put in here because by the time we do path selection, we don't have access to the
    /// consensus and so we have to keep our fallback ready.
    #[getset(get = "pub")]
    fixed_window_params: FixedWindowParams,
    /// Congestion window parameters.
    #[getset(get_copy = "pub", get_mut = "pub")]
    cwnd_params: CongestionWindowParams,
    /// RTT calculation parameters.
    #[getset(get = "pub")]
    rtt_params: RoundTripEstimatorParams,
}
/// Builder for [`CongestionControlParams`].
#[derive(Clone, Default)]
pub struct CongestionControlParamsBuilder {
    /// The configured `alg` value.
    alg: Option<Algorithm>,
    /// The configured `fixed_window_params` value.
    fixed_window_params: Option<FixedWindowParams>,
    /// The configured `cwnd_params` value.
    cwnd_params: Option<CongestionWindowParams>,
    /// The configured `rtt_params` value.
    rtt_params: Option<RoundTripEstimatorParams>,
}

impl CongestionControlParamsBuilder {
    /// Set the [`CongestionControlParams::alg`] value.
    pub fn alg(&mut self, value: Algorithm) -> &mut Self {
        self.alg = Some(value);
        self
    }

    /// Set the [`CongestionControlParams::fixed_window_params`] value.
    pub fn fixed_window_params(&mut self, value: FixedWindowParams) -> &mut Self {
        self.fixed_window_params = Some(value);
        self
    }

    /// Set the [`CongestionControlParams::cwnd_params`] value.
    pub fn cwnd_params(&mut self, value: CongestionWindowParams) -> &mut Self {
        self.cwnd_params = Some(value);
        self
    }

    /// Set the [`CongestionControlParams::rtt_params`] value.
    pub fn rtt_params(&mut self, value: RoundTripEstimatorParams) -> &mut Self {
        self.rtt_params = Some(value);
        self
    }

    /// Build the parameters, requiring every field to be set.
    pub fn build(&self) -> Result<CongestionControlParams, ConfigBuildError> {
        Ok(CongestionControlParams {
            alg: required_field(self.alg.clone(), "alg")?,
            fixed_window_params: required_field(self.fixed_window_params, "fixed_window_params")?,
            cwnd_params: required_field(self.cwnd_params, "cwnd_params")?,
            rtt_params: required_field(self.rtt_params.clone(), "rtt_params")?,
        })
    }
}

impl CongestionControlParams {
    /// Return a fresh, default builder.
    pub fn builder() -> CongestionControlParamsBuilder {
        CongestionControlParamsBuilder::default()
    }
}

impl tor_config::load::Builder for CongestionControlParamsBuilder {
    type Built = CongestionControlParams;

    fn build(&self) -> Result<CongestionControlParams, ConfigBuildError> {
        CongestionControlParamsBuilder::build(self)
    }
}

impl CongestionControlParams {
    /// Return true iff congestion control is enabled that is the algorithm is anything other than
    /// the fixed window SENDMEs.
    ///
    /// C-tor ref: congestion_control_enabled()
    pub(crate) fn is_enabled(&self) -> bool {
        !matches!(self.alg(), Algorithm::FixedWindow(_))
    }

    /// Make these parameters to use the fallback algorithm. This can't be reversed.
    pub(crate) fn use_fallback_alg(&mut self) {
        self.alg = Algorithm::FixedWindow(self.fixed_window_params);
    }

    /// Override the `cc_sendme_inc` value in these parameters with the value provided
    /// in `sendme_inc`.
    pub fn override_sendme_inc(&mut self, sendme_inc: u8) {
        self.cwnd_params.set_sendme_inc(sendme_inc);
    }
}

/// Return true iff the given sendme increment is valid with regards to the value in the circuit
/// parameters that is taken from the consensus.
#[allow(clippy::needless_bool)] // it's clearer this way
pub(crate) fn is_sendme_inc_valid(inc: u8, params: &CongestionControlParams) -> bool {
    // A consensus value of 1 would allow this sendme increment to be 0 and thus
    // we have to special case it before evaluating.
    if inc == 0 {
        return false;
    }
    let inc_consensus = params.cwnd_params().sendme_inc();
    // See prop324 section 10.3
    if inc > inc_consensus.saturating_add(1) || inc < inc_consensus.saturating_sub(1) {
        return false;
    }
    true
}

#[cfg(test)]
mod test {
    use crate::{
        ccparams::is_sendme_inc_valid, congestion::test_utils::params::build_cc_vegas_params,
    };

    /// Require the expected first missing field from a parameter builder.
    fn missing_field<T>(result: Result<T, tor_config::ConfigBuildError>, expected: &str) {
        match result {
            Err(tor_config::ConfigBuildError::MissingField { field }) => {
                assert_eq!(field, expected);
            }
            _ => panic!("expected missing field {expected}"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn required_fields_remain_required_and_ordered() {
        use super::*;

        let fixed_defaults = FixedWindowParams::defaults_for_tests();
        let mut fixed = FixedWindowParams::builder();
        missing_field(fixed.build(), "circ_window_start");
        fixed.circ_window_start(fixed_defaults.circ_window_start());
        missing_field(fixed.build(), "circ_window_min");
        fixed.circ_window_min(fixed_defaults.circ_window_min());
        missing_field(fixed.build(), "circ_window_max");
        fixed.circ_window_max(fixed_defaults.circ_window_max());
        let saved = fixed.clone();
        fixed.circ_window_start(fixed_defaults.circ_window_start() + 1);
        assert_eq!(
            tor_config::load::Builder::build(&fixed)
                .unwrap()
                .circ_window_start(),
            fixed_defaults.circ_window_start() + 1
        );
        assert_eq!(
            saved.build().unwrap().circ_window_start(),
            fixed_defaults.circ_window_start()
        );

        let vegas_defaults = VegasParams::defaults_for_tests();
        let mut vegas = VegasParams::builder();
        missing_field(vegas.build(), "cell_in_queue_params");
        vegas.cell_in_queue_params(*vegas_defaults.cell_in_queue_params());
        missing_field(vegas.build(), "ss_cwnd_max");
        vegas.ss_cwnd_max(vegas_defaults.ss_cwnd_max());
        missing_field(vegas.build(), "cwnd_full_gap");
        vegas.cwnd_full_gap(vegas_defaults.cwnd_full_gap());
        missing_field(vegas.build(), "cwnd_full_min_pct");
        vegas.cwnd_full_min_pct(*vegas_defaults.cwnd_full_min_pct());
        missing_field(vegas.build(), "cwnd_full_per_cwnd");
        vegas.cwnd_full_per_cwnd(vegas_defaults.cwnd_full_per_cwnd());
        assert_eq!(
            tor_config::load::Builder::build(&vegas)
                .unwrap()
                .cwnd_full_gap(),
            vegas_defaults.cwnd_full_gap()
        );

        let rtt_defaults = RoundTripEstimatorParams::defaults_for_tests();
        let mut rtt = RoundTripEstimatorParams::builder();
        missing_field(rtt.build(), "ewma_cwnd_pct");
        rtt.ewma_cwnd_pct(*rtt_defaults.ewma_cwnd_pct());
        missing_field(rtt.build(), "ewma_max");
        rtt.ewma_max(rtt_defaults.ewma_max());
        missing_field(rtt.build(), "ewma_ss_max");
        rtt.ewma_ss_max(rtt_defaults.ewma_ss_max());
        missing_field(rtt.build(), "rtt_reset_pct");
        rtt.rtt_reset_pct(*rtt_defaults.rtt_reset_pct());
        assert_eq!(
            tor_config::load::Builder::build(&rtt).unwrap().ewma_max(),
            rtt_defaults.ewma_max()
        );

        let window_defaults = CongestionWindowParams::defaults_for_tests();
        let mut window = CongestionWindowParams::builder();
        missing_field(window.build(), "cwnd_init");
        window.cwnd_init(window_defaults.cwnd_init());
        missing_field(window.build(), "cwnd_inc_pct_ss");
        window.cwnd_inc_pct_ss(*window_defaults.cwnd_inc_pct_ss());
        missing_field(window.build(), "cwnd_inc");
        window.cwnd_inc(window_defaults.cwnd_inc());
        missing_field(window.build(), "cwnd_inc_rate");
        window.cwnd_inc_rate(window_defaults.cwnd_inc_rate());
        missing_field(window.build(), "cwnd_min");
        window.cwnd_min(window_defaults.cwnd_min());
        missing_field(window.build(), "cwnd_max");
        window.cwnd_max(window_defaults.cwnd_max());
        missing_field(window.build(), "sendme_inc");
        window.sendme_inc(window_defaults.sendme_inc());
        assert_eq!(
            tor_config::load::Builder::build(&window)
                .unwrap()
                .sendme_inc(),
            window_defaults.sendme_inc()
        );

        let mut control = CongestionControlParams::builder();
        missing_field(control.build(), "alg");
        control.alg(Algorithm::FixedWindow(fixed_defaults));
        missing_field(control.build(), "fixed_window_params");
        control.fixed_window_params(fixed_defaults);
        missing_field(control.build(), "cwnd_params");
        control.cwnd_params(window_defaults);
        missing_field(control.build(), "rtt_params");
        control.rtt_params(rtt_defaults.clone());
        assert_eq!(
            tor_config::load::Builder::build(&control)
                .unwrap()
                .rtt_params()
                .ewma_max(),
            rtt_defaults.ewma_max()
        );
    }

    #[test]
    fn test_sendme_inc_valid() {
        let params = build_cc_vegas_params();
        let ref_inc = params.cwnd_params().sendme_inc();

        // In range.
        assert!(is_sendme_inc_valid(ref_inc, &params));
        assert!(is_sendme_inc_valid(ref_inc + 1, &params));
        assert!(is_sendme_inc_valid(ref_inc - 1, &params));
        // Out of range.
        assert!(!is_sendme_inc_valid(0, &params));
        assert!(!is_sendme_inc_valid(ref_inc + 2, &params));
        assert!(!is_sendme_inc_valid(ref_inc - 2, &params));
    }
}
