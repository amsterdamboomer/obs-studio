//! Windows GPU detection from DXGI adapters (obs-rust/obs-studio#942).

use obs_gpu_status::dxgi::{AdapterDesc, DXGI_ADAPTER_FLAG_SOFTWARE, devices_from_dxgi};
use obs_gpu_status::{GpuDevice, State, Vendor, classify};

fn adapter(description: &str, vendor_id: u32, device_id: u32, flags: u32) -> AdapterDesc {
    AdapterDesc {
        description: description.into(),
        vendor_id,
        device_id,
        flags,
    }
}

#[test]
fn amd_intel_and_nvidia_adapters_are_gpus() {
    let got = devices_from_dxgi(&[
        adapter("AMD Radeon RX 7900 XTX", 0x1002, 0x744c, 0),
        adapter("Intel(R) UHD Graphics 770", 0x8086, 0x4680, 0),
        adapter("NVIDIA GeForce RTX 3060", 0x10de, 0x2504, 0),
    ]);
    assert_eq!(
        got,
        vec![
            GpuDevice {
                vendor: Vendor::Amd,
                device_id: Some("744c".into()),
                name: Some("AMD Radeon RX 7900 XTX".into()),
                render_node: true,
            },
            GpuDevice {
                vendor: Vendor::Intel,
                device_id: Some("4680".into()),
                name: Some("Intel(R) UHD Graphics 770".into()),
                render_node: true,
            },
            GpuDevice {
                vendor: Vendor::Nvidia,
                device_id: Some("2504".into()),
                name: Some("NVIDIA GeForce RTX 3060".into()),
                render_node: true,
            },
        ]
    );
}

#[test]
fn software_and_microsoft_adapters_are_not_gpus() {
    let got = devices_from_dxgi(&[
        adapter(
            "Microsoft Basic Render Driver",
            0x1414,
            0x8c,
            DXGI_ADAPTER_FLAG_SOFTWARE,
        ),
        adapter("Microsoft Hyper-V Video", 0x1414, 0x5353, 0),
        adapter(
            "Some software rasterizer",
            0x10de,
            0x1,
            DXGI_ADAPTER_FLAG_SOFTWARE,
        ),
        adapter("Virtio GPU", 0x1af4, 0x1050, 0),
    ]);
    assert!(got.is_empty(), "{got:?}");
}

#[test]
fn amd_only_machine_on_basic_render_is_yellow_not_red() {
    // The #942 bug: no nvidia-smi on an AMD-only machine meant no GPUs and red.
    let devices = devices_from_dxgi(&[
        adapter("AMD Radeon RX 6600", 0x1002, 0x73ff, 0),
        adapter(
            "Microsoft Basic Render Driver",
            0x1414,
            0x8c,
            DXGI_ADAPTER_FLAG_SOFTWARE,
        ),
    ]);
    assert_eq!(
        classify(Some("Microsoft Basic Render Driver"), &devices, None),
        State::Yellow
    );
}

#[test]
fn trailing_spaces_and_nuls_are_trimmed_from_names() {
    let got = devices_from_dxgi(&[adapter("Intel(R) Arc(TM) A770  \0\0", 0x8086, 0x56a0, 0)]);
    assert_eq!(got[0].name.as_deref(), Some("Intel(R) Arc(TM) A770"));
}
