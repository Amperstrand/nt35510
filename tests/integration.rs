use std::collections::{HashMap, HashSet};

use embedded_display_controller::dsi::{DsiHostCtrlIo, DsiReadCommand, DsiWriteCommand};
use embedded_hal::delay::DelayNs;
use embedded_hal_mock::eh1::delay::NoopDelay;
use nt35510::{
    ColorFormat, ColorMap, Error, Mode, Nt35510, Nt35510Config, NT35510_CMD_B0, NT35510_CMD_B1,
    NT35510_CMD_B2, NT35510_CMD_B3, NT35510_CMD_B5, NT35510_CMD_B6, NT35510_CMD_B7, NT35510_CMD_B8,
    NT35510_CMD_B9, NT35510_CMD_BA, NT35510_CMD_BC, NT35510_CMD_BD, NT35510_CMD_BE, NT35510_CMD_BF,
    NT35510_CMD_CASET, NT35510_CMD_CC, NT35510_CMD_COLMOD, NT35510_CMD_DISPOFF, NT35510_CMD_DISPON,
    NT35510_CMD_MADCTL, NT35510_CMD_RAMWR, NT35510_CMD_RASET, NT35510_CMD_RDID1, NT35510_CMD_RDID2,
    NT35510_CMD_SETEXTC, NT35510_CMD_SLPIN, NT35510_CMD_SLPOUT, NT35510_CMD_TEEON,
    NT35510_CMD_WRCABC, NT35510_CMD_WRCABCMB, NT35510_CMD_WRCTRLD, NT35510_CMD_WRDISBV,
    NT35510_COLMOD_RGB565, NT35510_COLMOD_RGB888,
};

#[derive(Debug, Default)]
struct MockDsiHost {
    writes: Vec<(u8, Vec<u8>)>,
    read_responses: HashMap<u8, Vec<u8>>,
    read_errors: HashSet<u8>,
    write_kinds: Vec<u8>,
}

impl MockDsiHost {
    fn with_read_response(mut self, cmd: u8, data: &[u8]) -> Self {
        self.read_responses.insert(cmd, data.to_vec());
        self
    }

    fn with_read_error(mut self, cmd: u8) -> Self {
        self.read_errors.insert(cmd);
        self
    }
}

impl DsiHostCtrlIo for MockDsiHost {
    type Error = ();

    fn write(&mut self, cmd: DsiWriteCommand) -> Result<(), ()> {
        self.write_kinds.push(cmd.discriminant());
        match cmd {
            DsiWriteCommand::DcsShortP0 { arg } => self.writes.push((arg, Vec::new())),
            DsiWriteCommand::DcsShortP1 { arg, data } => self.writes.push((arg, vec![data])),
            DsiWriteCommand::DcsLongWrite { arg, data } => self.writes.push((arg, data.to_vec())),
            DsiWriteCommand::GenericLongWrite { arg, data } => {
                self.writes.push((arg, data.to_vec()))
            }
            DsiWriteCommand::SetMaximumReturnPacketSize(size) => {
                self.writes.push((0x37, size.to_be_bytes().to_vec()))
            }
            DsiWriteCommand::GenericShortP0 => self.writes.push((0x03, Vec::new())),
            DsiWriteCommand::GenericShortP1 => self.writes.push((0x13, Vec::new())),
            DsiWriteCommand::GenericShortP2 => self.writes.push((0x23, Vec::new())),
        }
        Ok(())
    }

    fn read(&mut self, cmd: DsiReadCommand, buf: &mut [u8]) -> Result<(), ()> {
        let DsiReadCommand::DcsShort { arg } = cmd else {
            return Err(());
        };

        if self.read_errors.contains(&arg) {
            return Err(());
        }

        let response = self.read_responses.get(&arg).cloned().unwrap_or_default();
        for (dst, src) in buf.iter_mut().zip(response.into_iter()) {
            *dst = src;
        }
        Ok(())
    }
}

#[derive(Default)]
struct RecordingDelay {
    inner: NoopDelay,
    calls_us: Vec<u32>,
}

impl DelayNs for RecordingDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.inner.delay_ns(ns);
    }

    fn delay_us(&mut self, us: u32) {
        self.calls_us.push(us);
        self.inner.delay_us(us);
    }

    fn delay_ms(&mut self, ms: u32) {
        self.inner.delay_ms(ms);
    }
}

fn default_config() -> Nt35510Config {
    Nt35510Config::default()
}

fn expected_init_sequence_rgb888_portrait() -> Vec<(u8, Vec<u8>)> {
    vec![
        (NT35510_CMD_SETEXTC, vec![0x55, 0xAA, 0x52, 0x08, 0x01]),
        (NT35510_CMD_B0, vec![0x03, 0x03, 0x03]),
        (NT35510_CMD_B6, vec![0x46, 0x46, 0x46]),
        (NT35510_CMD_B1, vec![0x03, 0x03, 0x03]),
        (NT35510_CMD_B7, vec![0x36, 0x36, 0x36]),
        (NT35510_CMD_B2, vec![0x00, 0x00, 0x02]),
        (NT35510_CMD_B8, vec![0x26, 0x26, 0x26]),
        (NT35510_CMD_BF, vec![0x01]),
        (NT35510_CMD_B3, vec![0x09, 0x09, 0x09]),
        (NT35510_CMD_B9, vec![0x36, 0x36, 0x36]),
        (NT35510_CMD_B5, vec![0x08, 0x08, 0x08]),
        (NT35510_CMD_BA, vec![0x26, 0x26, 0x26]),
        (NT35510_CMD_BC, vec![0x00, 0x80, 0x00]),
        (NT35510_CMD_BD, vec![0x00, 0x80, 0x00]),
        (NT35510_CMD_BE, vec![0x00, 0x50]),
        (NT35510_CMD_SETEXTC, vec![0x55, 0xAA, 0x52, 0x08, 0x00]),
        (NT35510_CMD_B1, vec![0xFC, 0x00]),
        (NT35510_CMD_B6, vec![0x03]),
        (NT35510_CMD_B5, vec![0x51]),
        (NT35510_CMD_B7, vec![0x00, 0x00]),
        (NT35510_CMD_B8, vec![0x01, 0x02, 0x02, 0x02]),
        (NT35510_CMD_BC, vec![0x00, 0x00, 0x00]),
        (NT35510_CMD_CC, vec![0x03, 0x00, 0x00]),
        (NT35510_CMD_BA, vec![0x01]),
        (NT35510_CMD_TEEON, vec![0x00]),
        (NT35510_CMD_COLMOD, vec![NT35510_COLMOD_RGB888]),
        (NT35510_CMD_MADCTL, vec![0x00]),
        (NT35510_CMD_CASET, vec![0x00, 0x00, 0x01, 0xDF]),
        (NT35510_CMD_RASET, vec![0x00, 0x00, 0x03, 0x1F]),
        (NT35510_CMD_SLPOUT, vec![0x00]),
        (NT35510_CMD_COLMOD, vec![NT35510_COLMOD_RGB888]),
        (NT35510_CMD_WRDISBV, vec![0x7F]),
        (NT35510_CMD_WRCTRLD, vec![0x2C]),
        (NT35510_CMD_WRCABC, vec![0x02]),
        (NT35510_CMD_WRCABCMB, vec![0xFF]),
        (NT35510_CMD_DISPON, vec![0x00]),
        (NT35510_CMD_RAMWR, vec![0x00]),
    ]
}

fn init_panel(config: Nt35510Config) -> (MockDsiHost, RecordingDelay) {
    let mut host = MockDsiHost::default();
    let mut delay = RecordingDelay::default();
    let mut panel = Nt35510::new();
    let result = panel.init_with_config(&mut host, &mut delay, config);
    assert_eq!(result, Ok(()));
    (host, delay)
}

#[test]
fn nt35510_config_validation() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default();
    let mut delay = RecordingDelay::default();

    let err = panel.init_with_config(
        &mut host,
        &mut delay,
        Nt35510Config {
            cols: 0,
            ..default_config()
        },
    );
    assert_eq!(err, Err(Error::InvalidDimensions));

    let err = panel.init_with_config(
        &mut host,
        &mut delay,
        Nt35510Config {
            rows: 0,
            ..default_config()
        },
    );
    assert_eq!(err, Err(Error::InvalidDimensions));

    let ok = panel.init_with_config(&mut host, &mut delay, default_config());
    assert_eq!(ok, Ok(()));
}

#[test]
fn write_reg_routing_observed_through_init_sequence() {
    let (host, _) = init_panel(default_config());

    assert!(host
        .write_kinds
        .contains(&DsiWriteCommand::DcsShortP1 { arg: 0, data: 0 }.discriminant()));
    assert!(host
        .write_kinds
        .contains(&DsiWriteCommand::DcsLongWrite { arg: 0, data: &[] }.discriminant()));

    let single_byte_bf = host
        .writes
        .iter()
        .position(|(cmd, data)| *cmd == NT35510_CMD_BF && data == &vec![0x01]);
    assert!(single_byte_bf.is_some());
}

#[test]
fn probe_accepts_expected_id2() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default().with_read_response(NT35510_CMD_RDID2, &[0x80]);
    assert_eq!(panel.probe(&mut host), Ok(()));
}

#[test]
fn probe_falls_back_to_expected_id1_when_id2_read_fails() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default()
        .with_read_error(NT35510_CMD_RDID2)
        .with_read_response(NT35510_CMD_RDID1, &[0x00]);
    assert_eq!(panel.probe(&mut host), Ok(()));
}

#[test]
fn probe_rejects_wrong_id() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default().with_read_response(NT35510_CMD_RDID2, &[0x42]);
    assert_eq!(panel.probe(&mut host), Err(Error::ProbeMismatch(0x42)));
}

#[test]
fn probe_reports_dsi_read_error() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default()
        .with_read_error(NT35510_CMD_RDID2)
        .with_read_error(NT35510_CMD_RDID1);
    assert_eq!(panel.probe(&mut host), Err(Error::DsiRead));
}

#[test]
fn id_matches_handles_expected_and_wrong_ids() {
    let mut panel = Nt35510::new();
    let mut id2_host = MockDsiHost::default().with_read_response(NT35510_CMD_RDID2, &[0x80]);
    assert_eq!(panel.id_matches(&mut id2_host), Ok(true));

    let mut id1_host = MockDsiHost::default()
        .with_read_error(NT35510_CMD_RDID2)
        .with_read_response(NT35510_CMD_RDID1, &[0x00]);
    assert_eq!(panel.id_matches(&mut id1_host), Ok(true));

    let mut wrong_host = MockDsiHost::default().with_read_response(NT35510_CMD_RDID2, &[0x42]);
    assert_eq!(panel.id_matches(&mut wrong_host), Ok(false));
}

#[test]
fn init_with_config_rgb888_portrait_sends_full_ordered_sequence() {
    let (host, delay) = init_panel(default_config());
    assert_eq!(host.writes, expected_init_sequence_rgb888_portrait());
    assert_eq!(delay.calls_us, vec![200_000, 120_000]);
}

#[test]
fn color_format_variants_set_expected_colmod_values() {
    let (rgb888_host, _) = init_panel(default_config());
    let rgb888_colmods: Vec<_> = rgb888_host
        .writes
        .iter()
        .filter(|(cmd, _)| *cmd == NT35510_CMD_COLMOD)
        .map(|(_, data)| data[0])
        .collect();
    assert_eq!(rgb888_colmods, vec![0x77, 0x77]);

    let (rgb565_host, _) = init_panel(Nt35510Config {
        color_format: ColorFormat::Rgb565,
        ..default_config()
    });
    let rgb565_colmods: Vec<_> = rgb565_host
        .writes
        .iter()
        .filter(|(cmd, _)| *cmd == NT35510_CMD_COLMOD)
        .map(|(_, data)| data[0])
        .collect();
    assert_eq!(rgb565_colmods, vec![0x77, 0x77, 0x55]);
}

#[test]
fn orientation_variants_set_expected_madctl() {
    let (portrait_host, _) = init_panel(default_config());
    let portrait = portrait_host
        .writes
        .iter()
        .find(|(cmd, _)| *cmd == NT35510_CMD_MADCTL)
        .map(|(_, data)| data[0]);
    assert_eq!(portrait, Some(0x00));

    let (landscape_host, _) = init_panel(Nt35510Config {
        mode: Mode::Landscape,
        ..default_config()
    });
    let landscape = landscape_host
        .writes
        .iter()
        .find(|(cmd, _)| *cmd == NT35510_CMD_MADCTL)
        .map(|(_, data)| data[0]);
    assert_eq!(landscape, Some(0x60));
}

#[test]
fn color_map_variants_set_expected_madctl_bit() {
    let (rgb_host, _) = init_panel(default_config());
    let rgb = rgb_host
        .writes
        .iter()
        .find(|(cmd, _)| *cmd == NT35510_CMD_MADCTL)
        .map(|(_, data)| data[0]);
    assert_eq!(rgb, Some(0x00));

    let (bgr_host, _) = init_panel(Nt35510Config {
        color_map: ColorMap::Bgr,
        ..default_config()
    });
    let bgr = bgr_host
        .writes
        .iter()
        .find(|(cmd, _)| *cmd == NT35510_CMD_MADCTL)
        .map(|(_, data)| data[0]);
    assert_eq!(bgr, Some(0x08));
}

#[test]
fn reinit_guard_skips_second_init_sequence() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default();
    let mut delay = RecordingDelay::default();

    assert_eq!(
        panel.init_with_config(&mut host, &mut delay, default_config()),
        Ok(())
    );
    let first_len = host.writes.len();
    let first_delays = delay.calls_us.clone();

    assert_eq!(
        panel.init_with_config(&mut host, &mut delay, default_config()),
        Ok(())
    );
    assert_eq!(host.writes.len(), first_len);
    assert_eq!(delay.calls_us, first_delays);
}

#[test]
fn sleep_in_clears_initialized_and_allows_reinit() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default();
    let mut delay = RecordingDelay::default();

    assert_eq!(
        panel.init_with_config(&mut host, &mut delay, default_config()),
        Ok(())
    );
    let init_len = host.writes.len();

    assert_eq!(panel.sleep_in(&mut host, &mut delay), Ok(()));
    assert_eq!(host.writes[init_len], (NT35510_CMD_DISPOFF, vec![0x00]));
    assert_eq!(host.writes[init_len + 1], (NT35510_CMD_SLPIN, vec![0x00]));

    assert_eq!(
        panel.init_with_config(&mut host, &mut delay, default_config()),
        Ok(())
    );
    assert_eq!(host.writes.len(), init_len * 2 + 2);
    assert_eq!(
        delay.calls_us,
        vec![200_000, 120_000, 120_000, 200_000, 120_000]
    );
}

#[test]
fn brightness_and_backlight_commands_are_sent() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default();

    assert_eq!(panel.set_brightness(&mut host, 0xFF), Ok(()));
    assert_eq!(panel.set_backlight(&mut host, true), Ok(()));
    assert_eq!(panel.set_backlight(&mut host, false), Ok(()));

    assert_eq!(
        host.writes,
        vec![
            (NT35510_CMD_WRDISBV, vec![0xFF]),
            (NT35510_CMD_WRCTRLD, vec![0x2C]),
            (NT35510_CMD_WRCTRLD, vec![0x00]),
        ]
    );
}

#[test]
fn error_display_messages_are_human_readable() {
    assert!(format!("{}", Error::DsiRead).contains("DSI read"));
    assert!(format!("{}", Error::ProbeMismatch(0x42)).contains("0x42"));
    assert!(format!("{}", Error::InvalidDimensions).contains("non-zero"));
}

#[test]
fn convenience_methods_match_expected_configs() {
    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default();
    let mut delay = RecordingDelay::default();
    assert_eq!(panel.init(&mut host, &mut delay), Ok(()));
    assert_eq!(host.writes, expected_init_sequence_rgb888_portrait());

    let mut panel = Nt35510::new();
    let mut host = MockDsiHost::default();
    let mut delay = RecordingDelay::default();
    assert_eq!(
        panel.init_rgb565(&mut host, &mut delay, Mode::Landscape, ColorMap::Bgr),
        Ok(())
    );
    let madctl = host
        .writes
        .iter()
        .find(|(cmd, _)| *cmd == NT35510_CMD_MADCTL)
        .map(|(_, data)| data[0]);
    let colmods: Vec<_> = host
        .writes
        .iter()
        .filter(|(cmd, _)| *cmd == NT35510_CMD_COLMOD)
        .map(|(_, data)| data[0])
        .collect();
    assert_eq!(madctl, Some(0x68));
    assert_eq!(
        colmods,
        vec![
            NT35510_COLMOD_RGB888,
            NT35510_COLMOD_RGB888,
            NT35510_COLMOD_RGB565
        ]
    );
}

#[test]
fn default_impls_match_documented_values() {
    assert_eq!(
        Nt35510Config::default(),
        Nt35510Config {
            mode: Mode::Portrait,
            color_map: ColorMap::Rgb,
            color_format: ColorFormat::Rgb888,
            cols: 480,
            rows: 800,
        }
    );

    let lhs = Nt35510::default();
    let rhs = Nt35510::new();
    let mut lhs_host = MockDsiHost::default();
    let mut rhs_host = MockDsiHost::default();
    let mut lhs_delay = RecordingDelay::default();
    let mut rhs_delay = RecordingDelay::default();
    let mut lhs = lhs;
    let mut rhs = rhs;
    assert_eq!(lhs.init(&mut lhs_host, &mut lhs_delay), Ok(()));
    assert_eq!(rhs.init(&mut rhs_host, &mut rhs_delay), Ok(()));
    assert_eq!(lhs_host.writes, rhs_host.writes);
}

#[test]
fn new_has_no_side_effects_before_init() {
    let _panel = Nt35510::new();
    let host = MockDsiHost::default();
    assert!(host.writes.is_empty());
}
