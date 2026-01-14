use std::fs;
use std::io::ErrorKind;
use std::sync::LazyLock;
use xxhash_rust::const_xxh3::xxh3_64;

// Records of various DMCHDC hashes and mods (Not complete, need GOG)
pub enum File {
    // HD Collection
    DMC1,
    DMC2,
    DMC3,
    // Mods (Probably not going to add every DDMK/Crimson Version, only from the time of writing and onwards)
    Eva,
    Lucia,
    Mary,
    Crimson,
}

pub struct VersionInformation {
    hash: u64,
    valid_for_use: bool,
    description: &'static str,
}

static EMPTY: LazyLock<Vec<VersionInformation>> = LazyLock::new(Vec::new);

static DMC1_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![
        VersionInformation {
            hash: 16596094990179088469,
            valid_for_use: true,
            description: "DDMK Patched DMC1",
        },
        VersionInformation {
            hash: 10860670779859874529,
            valid_for_use: true,
            description: "Version #1 DMC1",
        },
        VersionInformation {
            hash: 342337984247752146,
            valid_for_use: true,
            description: "Version #2 DMC1",
        },
        VersionInformation {
            hash: 6932768196842012018,
            valid_for_use: false,
            description: "Latest DMC1",
        },
    ]
});

static DMC2_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![
        VersionInformation {
            hash: 8566769405802122008,
            valid_for_use: true,
            description: "DDMK Patched DMC2",
        },
        VersionInformation {
            hash: 4868173191699540308,
            valid_for_use: true,
            description: "Version #1 DMC2",
        },
        VersionInformation {
            hash: 5981905978386037807,
            valid_for_use: true,
            description: "Version #2 DMC2",
        },
        VersionInformation {
            hash: 7733538334450880217,
            valid_for_use: false,
            description: "Latest DMC2",
        },
    ]
});

static DMC3_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![
        VersionInformation {
            hash: 9031715114876197692,
            valid_for_use: true,
            description: "DDMK Patched DMC3",
        },
        VersionInformation {
            hash: 7198991379004446668,
            valid_for_use: true,
            description: "Crimson Patched DMC3",
        },
        VersionInformation {
            hash: 14598701335922013533,
            valid_for_use: true,
            description: "Version #1 DMC3",
        },
        VersionInformation {
            hash: 6772293939166567304,
            valid_for_use: true,
            description: "Version #2 DMC3",
        },
        VersionInformation {
            hash: 11219846177156872589,
            valid_for_use: false,
            description: "Latest DMC3",
        },
    ]
});

static DMC_LAUNCHER_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![
        VersionInformation {
            hash: 3012265650586028916,
            valid_for_use: true,
            description: "Crimson/DDMK Patched DMC Launcher",
        },
        VersionInformation {
            hash: 9711695139658080865,
            valid_for_use: true,
            description: "Version #1 DMC Launcher",
        },
        VersionInformation {
            hash: 14560228364278330367,
            valid_for_use: true,
            description: "Version #2 DMC Launcher",
        },
        VersionInformation {
            hash: 8868518716288212586,
            valid_for_use: true,
            description: "Latest DMC Launcher",
        },
    ]
});

static EVA_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![VersionInformation {
        hash: 2536699235936189826,
        valid_for_use: true,
        description: "2.7.3 DDMK - Eva",
    }]
});

static LUCIA_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![VersionInformation {
        hash: 16520636509798662806,
        valid_for_use: true,
        description: "2.7.3 DDMK - Lucia",
    }]
});

static MARY_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![VersionInformation {
        hash: 7087074874482460961,
        valid_for_use: true,
        description: "2.7.3 DDMK - Mary",
    }]
});

static CRIMSON_INFO: LazyLock<Vec<VersionInformation>> = LazyLock::new(|| {
    vec![VersionInformation {
        hash: 6027093939875741571,
        valid_for_use: true,
        description: "0.4 Crimson - Mary",
    }]
});

impl File {
    pub fn get_information(&self) -> &Vec<VersionInformation> {
        match self {
            File::DMC1 => &DMC1_INFO,
            File::DMC2 => &DMC2_INFO,
            File::DMC3 => &DMC3_INFO,

            File::Eva => &EVA_INFO,
            File::Lucia => &LUCIA_INFO,
            File::Mary => &MARY_INFO,
            File::Crimson => &CRIMSON_INFO,
        }
    }
}

pub(crate) fn is_file_valid(file_path: &str, expected_hash: u64) -> Result<(), std::io::Error> {
    let data = fs::read(file_path)?;
    if xxh3_64(&data) == expected_hash {
        Ok(())
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "File has invalid hash",
        ))
    }
}
