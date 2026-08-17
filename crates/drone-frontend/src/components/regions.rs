//! # Tactical Theaters
//!
//! The catalogue of map regions the mission selector can load. Each theater
//! carries its own sortie route; choosing one re-centres the map, swaps the
//! red waypoint pins and track, and restarts the convoy at that route's IP.
//!
//! Client-side by design: theaters are a VIEW concern. The `waypoints` query
//! is real server-side, and swapping these static routes for its result is a
//! follow-up that touches only `map.rs`.

/// One selectable theater.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TheaterId {
    Afghanistan,
    Syria,
    Libya,
    Pakistan,
    Iran,
    Iraq,
}

impl TheaterId {
    /// Every theater, in selector order. Afghanistan first — it is the default.
    pub const ALL: [TheaterId; 6] = [
        TheaterId::Afghanistan,
        TheaterId::Syria,
        TheaterId::Libya,
        TheaterId::Pakistan,
        TheaterId::Iran,
        TheaterId::Iraq,
    ];

    pub fn theater(self) -> &'static Theater {
        match self {
            TheaterId::Afghanistan => &AFGHANISTAN,
            TheaterId::Syria => &SYRIA,
            TheaterId::Libya => &LIBYA,
            TheaterId::Pakistan => &PAKISTAN,
            TheaterId::Iran => &IRAN,
            TheaterId::Iraq => &IRAQ,
        }
    }
}

impl Default for TheaterId {
    fn default() -> Self {
        TheaterId::Afghanistan
    }
}

/// A theater: where the map looks and the route the convoy flies there.
pub struct Theater {
    pub id: TheaterId,
    /// Selector label, e.g. "AFGHANISTAN".
    pub label: &'static str,
    /// AOR name shown in the map overlay, e.g. "KANDAHAR AOR".
    pub aor: &'static str,
    pub center: (f64, f64),
    pub zoom: u32,
    /// AOR ring radius, metres.
    pub aor_radius_m: f64,
    /// Sortie route, ingress → target run → egress, as (lat, lon).
    pub route: &'static [(f64, f64)],
}

/// Kandahar AOR: ingress NW, target run through the centre, egress SE.
static AFGHANISTAN: Theater = Theater {
    id: TheaterId::Afghanistan,
    label: "AFGHANISTAN",
    aor: "KANDAHAR AOR",
    center: (31.6289, 65.7372),
    zoom: 8,
    aor_radius_m: 150_000.0,
    route: &[
        (31.9800, 64.9000), (31.9100, 65.0600), (31.8300, 65.2100), (31.7600, 65.3500),
        (31.7000, 65.4900), (31.6600, 65.6200), (31.6289, 65.7372), (31.5900, 65.8500),
        (31.5400, 65.9600), (31.4800, 66.0600), (31.4100, 66.1500), (31.3400, 66.2300),
        (31.2600, 66.3100), (31.1800, 66.3900),
    ],
};

/// Euphrates corridor: Aleppo approach in the west, run east past Raqqa toward Deir ez-Zor.
static SYRIA: Theater = Theater {
    id: TheaterId::Syria,
    label: "SYRIA",
    aor: "RAQQA AOR",
    center: (35.9500, 38.9000),
    zoom: 7,
    aor_radius_m: 160_000.0,
    route: &[
        (36.3500, 37.6000), (36.2800, 37.9000), (36.2000, 38.2000), (36.1000, 38.4800),
        (36.0200, 38.7500), (35.9500, 38.9900), (35.8800, 39.2500), (35.7900, 39.5000),
        (35.6800, 39.7500), (35.5500, 40.0000), (35.4200, 40.2200), (35.3300, 40.4200),
    ],
};

/// Gulf of Sidra coast: Misrata approach, run east along the littoral past Sirte toward Benghazi.
static LIBYA: Theater = Theater {
    id: TheaterId::Libya,
    label: "LIBYA",
    aor: "SIRTE AOR",
    center: (31.2000, 17.5000),
    zoom: 7,
    aor_radius_m: 200_000.0,
    route: &[
        (32.3000, 15.2000), (32.1000, 15.6500), (31.8500, 16.0500), (31.6000, 16.4000),
        (31.3500, 16.7000), (31.2050, 16.5900), (31.1000, 17.1000), (30.9800, 17.6000),
        (30.9000, 18.1000), (30.9500, 18.7000), (31.1000, 19.2500), (31.3500, 19.7500),
        (31.7000, 20.0500),
    ],
};

/// FATA / Balochistan belt: Quetta approach, run north-east along the frontier toward Peshawar.
static PAKISTAN: Theater = Theater {
    id: TheaterId::Pakistan,
    label: "PAKISTAN",
    aor: "FRONTIER AOR",
    center: (32.0000, 69.8000),
    zoom: 7,
    aor_radius_m: 180_000.0,
    route: &[
        (30.1800, 66.9900), (30.6000, 67.4000), (31.0500, 67.8500), (31.5000, 68.3000),
        (31.9000, 68.7500), (32.3000, 69.2000), (32.7000, 69.6000), (33.1000, 70.0000),
        (33.5000, 70.4000), (33.8500, 70.8000), (34.0100, 71.5200),
    ],
};

/// Tehran approaches: run in from the west across the Alborz foothills, over the capital, egress east.
static IRAN: Theater = Theater {
    id: TheaterId::Iran,
    label: "IRAN",
    aor: "TEHRAN AOR",
    center: (35.6892, 51.3890),
    zoom: 8,
    aor_radius_m: 120_000.0,
    route: &[
        (35.9500, 50.3000), (35.9000, 50.5500), (35.8500, 50.8000), (35.8000, 51.0500),
        (35.7500, 51.2500), (35.6892, 51.3890), (35.6300, 51.5500), (35.5800, 51.7500),
        (35.5200, 51.9500), (35.4600, 52.1500), (35.4000, 52.3500), (35.3200, 52.5500),
    ],
};

/// Baghdad belt: Fallujah approach from the west, run over the capital, egress toward Baqubah.
static IRAQ: Theater = Theater {
    id: TheaterId::Iraq,
    label: "IRAQ",
    aor: "BAGHDAD AOR",
    center: (33.3152, 44.3661),
    zoom: 8,
    aor_radius_m: 120_000.0,
    route: &[
        (33.4500, 43.4000), (33.4200, 43.6500), (33.3900, 43.8800), (33.3600, 44.1000),
        (33.3300, 44.2500), (33.3152, 44.3661), (33.3600, 44.5500), (33.4200, 44.7000),
        (33.5000, 44.8500), (33.6000, 44.9800), (33.7000, 45.0800), (33.7500, 44.6300),
    ],
};
