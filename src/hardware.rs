use serde::{Deserialize, Serialize};

/// Hardware accelerator device family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeviceFamily {
    Gpu,
    Tpu,
    Npu,
    AiAsic,
    Cpu,
}

/// Hardware accelerator vendor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeviceVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Google,
    Qualcomm,
    Habana,
    Aws,
    Custom(String),
}

/// Compute framework availability flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceleratorFlags {
    #[serde(default)]
    pub cuda_available: bool,
    #[serde(default)]
    pub rocm_available: bool,
    #[serde(default)]
    pub metal_available: bool,
    #[serde(default)]
    pub vulkan_available: bool,
    #[serde(default)]
    pub oneapi_available: bool,
    #[serde(default)]
    pub tpu_available: bool,
    #[serde(default)]
    pub sycl_available: bool,
    #[serde(default)]
    pub openvino_available: bool,
    #[serde(default)]
    pub directml_available: bool,
}

/// Device health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum DeviceHealth {
    #[default]
    Ok,
    Degraded,
    Failed,
    Unknown,
}

/// Memory technology type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MemoryType {
    Gddr5,
    Gddr6,
    Gddr6x,
    Hbm2,
    Hbm2e,
    Hbm3,
    Lpddr4,
    Lpddr5,
    Lpddr5x,
    Ddr4,
    Ddr5,
    Unknown,
}

/// A detected hardware accelerator device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceleratorDevice {
    pub index: u32,
    pub name: String,
    pub vendor: DeviceVendor,
    pub family: DeviceFamily,
    /// Total VRAM in megabytes.
    #[serde(default)]
    pub vram_total_mb: u64,
    /// Used VRAM in megabytes.
    #[serde(default)]
    pub vram_used_mb: u64,
    /// Current utilization percentage (0-100).
    #[serde(default)]
    pub utilization_percent: u32,
    /// GPU temperature in Celsius.
    #[serde(default)]
    pub temperature_celsius: Option<f32>,
    #[serde(default)]
    pub driver_version: String,
    /// CUDA compute capability (e.g. "8.9").
    #[serde(default)]
    pub compute_capability: Option<String>,
    #[serde(default)]
    pub flags: AcceleratorFlags,
    /// Thermal design power in watts.
    #[serde(default)]
    pub power_watts: Option<f32>,
    /// Memory bandwidth in GB/s.
    #[serde(default)]
    pub memory_bandwidth_gbps: Option<f32>,
    /// Memory technology type.
    #[serde(default)]
    pub memory_type: Option<MemoryType>,
    /// Device health status.
    #[serde(default)]
    pub health: DeviceHealth,
}

/// Summary of all detected accelerators on the host.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcceleratorSummary {
    pub devices: Vec<AcceleratorDevice>,
    /// Total VRAM across all devices in megabytes.
    #[serde(default)]
    pub total_vram_mb: u64,
}

impl AcceleratorSummary {
    /// Filter devices by family.
    #[must_use]
    pub fn by_family(&self, family: DeviceFamily) -> Vec<&AcceleratorDevice> {
        self.devices.iter().filter(|d| d.family == family).collect()
    }

    /// Filter devices by vendor.
    #[must_use]
    pub fn by_vendor(&self, vendor: &DeviceVendor) -> Vec<&AcceleratorDevice> {
        self.devices
            .iter()
            .filter(|d| &d.vendor == vendor)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_family_serde_roundtrip() {
        for variant in [
            DeviceFamily::Gpu,
            DeviceFamily::Tpu,
            DeviceFamily::Npu,
            DeviceFamily::AiAsic,
            DeviceFamily::Cpu,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: DeviceFamily = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn device_vendor_serde_roundtrip() {
        for variant in [
            DeviceVendor::Nvidia,
            DeviceVendor::Amd,
            DeviceVendor::Intel,
            DeviceVendor::Apple,
            DeviceVendor::Google,
            DeviceVendor::Qualcomm,
            DeviceVendor::Habana,
            DeviceVendor::Aws,
            DeviceVendor::Custom("other".into()),
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let _back: DeviceVendor = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn accelerator_flags_default() {
        let f = AcceleratorFlags::default();
        assert!(!f.cuda_available);
        assert!(!f.rocm_available);
    }

    #[test]
    fn accelerator_device_serde_roundtrip() {
        let d = AcceleratorDevice {
            index: 0,
            name: "NVIDIA RTX 4090".into(),
            vendor: DeviceVendor::Nvidia,
            family: DeviceFamily::Gpu,
            vram_total_mb: 24576,
            vram_used_mb: 4096,
            utilization_percent: 45,
            temperature_celsius: Some(72.0),
            driver_version: "550.67".into(),
            compute_capability: Some("8.9".into()),
            flags: AcceleratorFlags {
                cuda_available: true,
                ..AcceleratorFlags::default()
            },
            power_watts: Some(450.0),
            memory_bandwidth_gbps: Some(1008.0),
            memory_type: Some(MemoryType::Gddr6x),
            health: DeviceHealth::Ok,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: AcceleratorDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "NVIDIA RTX 4090");
        assert_eq!(back.vram_total_mb, 24576);
        assert!(back.flags.cuda_available);
        assert_eq!(back.health, DeviceHealth::Ok);
        assert_eq!(back.memory_type, Some(MemoryType::Gddr6x));
    }

    #[test]
    fn accelerator_summary_by_family() {
        let summary = AcceleratorSummary {
            devices: vec![
                AcceleratorDevice {
                    index: 0,
                    name: "GPU 0".into(),
                    vendor: DeviceVendor::Nvidia,
                    family: DeviceFamily::Gpu,
                    vram_total_mb: 24576,
                    vram_used_mb: 0,
                    utilization_percent: 0,
                    temperature_celsius: None,
                    driver_version: String::new(),
                    compute_capability: None,
                    flags: AcceleratorFlags::default(),
                    power_watts: None,
                    memory_bandwidth_gbps: None,
                    memory_type: None,
                    health: DeviceHealth::default(),
                },
                AcceleratorDevice {
                    index: 1,
                    name: "NPU 0".into(),
                    vendor: DeviceVendor::Intel,
                    family: DeviceFamily::Npu,
                    vram_total_mb: 0,
                    vram_used_mb: 0,
                    utilization_percent: 0,
                    temperature_celsius: None,
                    driver_version: String::new(),
                    compute_capability: None,
                    flags: AcceleratorFlags::default(),
                    power_watts: None,
                    memory_bandwidth_gbps: None,
                    memory_type: None,
                    health: DeviceHealth::default(),
                },
            ],
            total_vram_mb: 24576,
        };
        assert_eq!(summary.by_family(DeviceFamily::Gpu).len(), 1);
        assert_eq!(summary.by_family(DeviceFamily::Npu).len(), 1);
        assert_eq!(summary.by_family(DeviceFamily::Tpu).len(), 0);
    }

    #[test]
    fn accelerator_summary_serde_roundtrip() {
        let s = AcceleratorSummary::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: AcceleratorSummary = serde_json::from_str(&json).unwrap();
        assert!(back.devices.is_empty());
    }
}
